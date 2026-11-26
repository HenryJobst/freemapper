# Program texts of freemapper, Finnish (ADR 0092). English is the reference: every other language
# has the same message ids and the same variables. Texts from other programs (operating system
# errors, parser messages) arrive in variables and stay as they are.

## The --lang switch

cli-lang-unknown = { $value } ei ole kieli, jolle freemapperilla on tekstit; saatavilla: { $available }

## init

cli-part-needs-osm = --part valitsee OSM-alueen osan ja vaatii valitsimen --osm; jos käytät valitsinta --place, hae ensin tunnus ja käytä sitten --osm <id> --part <n>
cli-reading-file = luetaan { $path }
cli-no-area-for-place = paikalle "{ $query }" ei löytynyt aluetta. Jos paikka koostuu useista erillisistä osista, valitse yksi valitsimilla --osm <id> --part <n> tai käytä valitsinta --geojson
cli-using-place = Käytetään paikkaa { $name } ({ $id })
cli-created-project = Projekti luotu: { $path }

## Cache

cli-no-cache-dir = tässä järjestelmässä ei ole välimuistikansiota; anna --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] Haettu 1 ruutu
       *[other] Haettu { $count } ruutua
    }
cli-wrote = Kirjoitettu { $path }
cli-kept-map = Säilytetty { $path } (siinä voi olla työtäsi)
cli-map-lacks-templates = Kartta ei viittaa vielä kuvapohjiin. Lisää ne OOM:ssa (Templates > Open template…) tai korvaa kartta valitsimella --force, joka säilyttää vanhan tiedostona .omap.bak.
cli-map-lacks-buildings = Kartalla ei ole vielä ALKIS-rakennuksia. Lisää ne korvaamalla kartta valitsimella --force, joka säilyttää vanhan tiedostona .omap.bak.
cli-map-lacks-canopies = Kartalta puuttuvat ALKIS-katokset ja autokatokset (522), jotka ovat nyt karttakohteita. Lisää ne ajamalla komento valitsimella --force, joka säilyttää vanhan kartan tiedostona .omap.bak.
cli-map-light = Pohjakartta: { $joined } viivaa yhdistetty, { $left } piilotettua tai liian pientä jätetty ehdokastasoille (mitään ei siirretty)
cli-map-light-kept = Ehdokastasoissa on nyt myös { $left } kohdetta tai viivan osaa, jotka tämä versio ottaa pois pohjakartalta (merkitty freemapper:generalized). Säilytetyllä kartalla ne ovat yhä, joten ne näkyvät kahdesti. Siivoa kartta korvaamalla se valitsimella --force (vanha säilyy tiedostona .omap.bak).
cli-map-old-candidates = Kartta viittaa yhä tiedostoon templates/candidates.omap; tämä ajo täytti sen kaikilla ehdokkailla. Ehdokkaita on nyt myös seitsemänä tasona, yksi tiedosto kullekin (templates/candidates-*.omap). Kartta luettelee ne, kun korvaat sen valitsimella --force (vanha säilyy tiedostona .omap.bak), tai voit lisätä ne itse OOM:ssa (Templates > Open template…).

## PDF

cli-pdf-without-credits = PDF ilman lähdemainintoja: ODbL ja dl-de/by-2-0 edellyttävät, että mainitset lähteet, kun luovutat kartan eteenpäin; katso ATTRIBUTION.txt
cli-wrote-pdf = Kirjoitettu { $path } (kartta ilman ehdokkaita)
cli-wrote-pdf-candidates = Kirjoitettu { $path } (tarkistustuloste: kartta kaikkine ehdokkaineen, paikkatarkasti; tarkista ne maastossa)
cli-wrote-training = Kirjoitettu { $map } ja { $pdf } (harjoituskartta: { $joined } yhdistetty, { $shifted } siirretty, { $left } jätetty pois tai katkaistu)
cli-training-free-ends = Viivojen vapaaksi jätetyt päät: { $ways } kohdassa, jossa polku kohtasi toisen polun, { $others } muiden viivojen kohdalla
cli-training-apart-left = harjoituskartalla on yhä { $length } mm (paperilla), jossa kohteiden väli jää liian pieneksi; tarkista se OOM:ssa
cli-kept-training = Säilytetty harjoituskartta { $path }, jotta ajo ei korvaa tulostettua tai eteenpäin annettua karttaa; poista se tai anna --force uusiaksesi sen

## OCAD

cli-ocad-kept = Säilytetty { $path } (siinä voi olla työtäsi; --force korvaa sen ja säilyttää vanhan tiedostona .ocd.bak)
cli-ocad-old-candidates = OCAD-kartta viittaa yhä tiedostoon { $path } taustakarttana; tämä ajo täytti sen kaikilla ehdokkailla. Ehdokkaita on nyt myös seitsemänä tasona, yksi tiedosto kullekin (-candidates-*.ocd). OCAD-kartta luettelee ne, kun korvaat sen valitsimella --force (vanha säilyy tiedostona .ocd.bak), tai voit lisätä ne itse OCADissa taustakarttoina.
cli-ocad-wrote = { $count ->
        [one] Kirjoitettu { $path } (1 kohde)
       *[other] Kirjoitettu { $path } ({ $count } kohdetta)
    }
cli-ocad-left-out = OCAD-kartalta jätettiin pois: { $skipped } kohdetta (ei OCAD-symbolia, ei aluetta tai OCADin piirtoalueen ±2 m ulkopuolella) ja { $dropped } rappeutunutta viivan osaa tai reikää.
cli-ocad-load-symbols = Valitse OCADissa Symbol > Load Colors and Symbols From… ja valitse virallinen symbolisarja{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] {" "}eli IOF:n koulukarttasarja 2019
       *[other] {" "}ISSprOM 2019
    }; se korvaa freemapperin paikkamerkkisymbolit numeron mukaan.

## select

cli-select-running = Alueen valinta kohteelle { $path }: paina sivulla "Valmis" tai Ctrl+C lopettaaksesi.
cli-select-no-browser = Selainta ei voitu avata ({ $error }); avaa yllä oleva osoite.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; ei aluetta, ei voi olla kartta-alue): { $place }
cli-find-hint = Luo projekti komennolla: freemapper init <dir> --osm { $id }
cli-find-no-area = Millään näistä paikoista ei ole aluetta, joka voisi olla kartta-alue. Piirrä alue komennolla: freemapper select <dir>
cli-find-nothing = paikkaa "{ $query }" ei löytynyt; kokeile toista nimeä tai piirrä alue komennolla: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = varoitus: kansion { $folder } sovitin jätetään pois: { $finding }
cli-adapter-note = varoitus: kansion { $folder } sovitin latautuu, mutta: { $finding }
cli-adapter-from-project = varoitus: projektikansion sovitin { $adapter } on käytössä; se voi kysellä näitä palvelimia: { $hosts }
cli-adapter-overlaps = varoitus: sovitin { $adapter } kattaa myös alueen { $others }; sen kohteet päätyvät siellä oleville kartoille
cli-adapter-dir-relative = varoitus: FREEMAPPER_ADAPTER_DIR ei ole absoluuttinen polku ({ $path }); se ohitetaan
cli-adapters-no-project = varoitus: { $path } ei ole kansio; projektin sovittimia ei luettu
cli-adapters-export-unknown = sovitinta { $adapter } ei kuvaa tämän ohjelman mikään manifesti, joten viemistä ei ole; `freemapper adapters list` näyttää tunnisteet
cli-adapters-export-taken = { $path } on olemassa eikä ole tyhjä; vie kansioon, jota ei vielä ole
cli-adapters-export-done = Sovittimen { $adapter } { $count } tiedostoa kirjoitettiin kansioon { $path }. Niissä on vain se, minkä manifesti kuvaa; ohjelman koodissa olevia osoitteita niissä ei ole.
cli-adapters-export-next = Muuta osoitteet tiedostossa manifest.toml, tarkista kansio komennolla `freemapper adapters check` ja siirrä se kansioon { $path }: seuraavasta käynnistyksestä alkaen se korvaa sisäänrakennetun sovittimen varoituksen kera. Kansion poistaminen palauttaa sisäänrakennetun sovittimen.
cli-adapters-overlaps = jakaa alueen kanssa: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  versio { $revision }  tarkistettu { $checked }  palvelimet: { $hosts }
cli-adapters-source-built-in = sisäänrakennettu
cli-adapters-source-user = omasta kansiostasi
cli-adapters-source-project = projektista
cli-adapters-never-checked = ei koskaan
cli-adapters-user-dir = Sovittimesi luetaan kansiosta { $path }
cli-adapters-no-user-dir = Tässä järjestelmässä ei ole kansiota sovittimillesi; FREEMAPPER_ADAPTER_DIR nimeää sellaisen

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper saa nyt kysellä myös palvelinta { $host } kohteelle { $name }. Merkintä on tiedostossa { $file }; `freemapper adapters disallow-host { $adapter } { $host }` peruu sen. Käynnissä oleva aluesivu tai tekoälyavustaja lukee merkinnän, kun se käynnistetään uudelleen
cli-allow-host-already = { $host } on jo sallittu kohteelle { $name }
cli-allow-host-declared = kohteen { $name } sovitin nimeää palvelimen { $host } itse; ei tarvitse sallia mitään
cli-allow-host-refused = palvelinta { $host } ei voi sallia: { $reason }
cli-allow-host-unknown-adapter = sisäänrakennettua sovitinta { $adapter } ei ole; sisäänrakennetut sovittimet ovat: { $adapters }
cli-allow-host-folder-adapter = { $adapter } on sovittimien kansiosi sovitin; lisää palvelin sen manifestin kohtaan `[net] hosts`
cli-allow-host-no-config = tässä järjestelmässä ei ole asetuskansiota sallittujen palvelimien tallentamiseen
cli-allow-host-write = tiedostoa { $file } ei voitu kirjoittaa: { $detail }
cli-allow-host-unreadable = tiedostoa { $file } ei voi lukea ({ $detail }); korjaa tai poista tiedosto ensin; mitään ei muutettu
cli-disallow-host-removed = freemapper ei enää kysele palvelinta { $host } kohteelle { $name }
cli-disallow-host-absent = palvelinta { $host } ei ollut sallittu kohteelle { $name }; ei mitään peruttavaa
cli-allowed-hosts-unreadable = varoitus: tiedostoa { $file } ei voitu lukea ({ $detail }); ylimääräisiä palvelimia ei sallita
cli-allowed-hosts-ignored = varoitus: { $file }: merkintä { $host } kohteelle { $adapter } ohitetaan: { $reason }
cli-allowed-hosts-no-adapter = freemapperilla ei ole tämännimistä sisäänrakennettua sovitinta
cli-allowed-host-redundant = varoitus: { $host } on nyt yksi palvelimista, jotka kohteen { $name } sovitin nimeää; merkintäsi tiedostossa { $file } poistettiin
cli-adapters-allowed = sinun sallimasi: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = sovitin { $adapter } ({ $name }), versio { $revision }, palvelimet: { $hosts }
cli-check-error = virhe: { $finding }
cli-check-warning = varoitus: { $finding }
cli-check-passed = { $folder } läpäisee tarkistuksen
cli-check-passed-warnings = { $folder } läpäisee tarkistuksen, { $count ->
        [one] yhdellä varoituksella
       *[other] { $count } varoituksella
    }
cli-check-failed = { $folder } ei läpäise tarkistusta
cli-check-no-folder = kansiota { $folder } ei ole
cli-check-live-ok = { $capability } ok: { $detail }
cli-check-live-failed = { $capability } EPÄONNISTUI: { $detail }
cli-check-live-not-asked = { $capability } ei kysytty: { $detail }
cli-check-live-box-outside = suorakulmio ei ole sovittimen { $adapter } rajojen sisällä; live-tarkistus kysyy palveluilta paikkaa, jonka sovitin kattaa
cli-check-live-cannot = live-tarkistusta ei voi ajaa: { $detail }
cli-check-detail-not-declared = sovitin ei ilmoita yhtään
cli-check-detail-stopped = aiempi osa epäonnistui
cli-check-detail-see-warnings = katso varoitukset
cli-check-detail-outside-dataset = suorakulmio on aineiston ulkopuolella
cli-check-detail-objects = { $count ->
        [one] yksi kohde
       *[other] { $count } kohdetta
    }
cli-check-detail-layers = { $layers } tasoa, { $count } kohdetta
cli-check-detail-trees = { $count ->
        [one] yksi puu
       *[other] { $count } puuta
    }
cli-check-detail-extract = { $label }, tiedot päivältä { $date }
cli-check-raster-orthophoto = ortokuva
cli-check-raster-terrain = maastomalli
cli-check-raster-surface = pintamalli
cli-check-raster-tiles = { $product }: { $count ->
        [one] yksi ruutu
       *[other] { $count } ruutua
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } ja { $last }

## Folder lock

cli-lock-held = toinen freemapper-ajo käyttää tätä kansiota (alkaen { $time }, käynnistäjä: { $who }). Odota sitä tai pysäytä se ensin.
cli-lock-who-cli = pääte
cli-lock-who-mcp = tekoälyagentti (MCP)
cli-lock-who-page = valintasivu
cli-lock-who-unknown = toinen ohjelma
cli-lock-failed = projektikansiota ei voitu lukita tätä ajoa varten: { $error }
cli-lock-stuck = tiellä on jokin muu kuin lukkotiedosto, eikä sitä voitu poistaa

## Run routes of the selection page

cli-run-bad-request = Ajoasetukset eivät kelpaa: { $error }
cli-run-no-project = Tässä kansiossa ei ole vielä tallennettua projektia. Tallenna ensin alue.
cli-run-already-running = Tälle projektille on jo käynnissä ajo ({ $job }). Odota sen päättymistä tai peruuta se ensin.
cli-run-disk-stops = Levytilaa ei ole tarpeeksi lataukselle: { $messages }
cli-run-busy-elsewhere = Toinen tämän ohjelman projekti ajaa parhaillaan putkea. Projektit jakavat latausvälimuistin, joten kerrallaan voi olla käynnissä vain yksi ajo. Yritä uudelleen, kun se on valmis.
cli-run-nothing-to-cancel = Mitään ajoa ei ole käynnissä, joten ei ole mitään peruttavaa.
cli-run-unknown-job = Tällä projektilla ei ole ajoa { $job }.
cli-status-too-fast = Tilakyselyjä tulee liian tiheästi. Kysy enintään viisi kertaa sekunnissa.
cli-open-folder-failed = Kansiota ei voitu avata ({ $error }).
cli-open-folder-no-opener = Tällä tietokoneella ei ole ohjelmaa kansioiden avaamiseen ({ $program }). Avaa kansio käsin.
cli-open-folder-missing = Projektikansiota ei ole vielä olemassa. Tallenna ensin alue.
cli-open-folder-refused = Kansion nimeä ei voi turvallisesti välittää tiedostonhallinnalle.
cli-save-running = Tälle projektille on käynnissä ajo. Tallenna uudelleen, kun se on valmis.
cli-save-unknown-kind = Tuntematon karttatyyppi "{ $kind }". Käytä arvoa schoolyard, sprint tai forest.
cli-save-scale-too-large = Mittakaava { $scale } on liian suuri.
cli-save-bad-content = Sisältöasetukset eivät kelpaa. Mitään ei tallennettu.
cli-map-image-none = Luo ensin kartta.
cli-map-image-gone = Tämä esikatselu ei ole enää saatavilla.
cli-map-image-running = Tälle projektille on käynnissä ajo. Esikatselun voi laskea, kun se on valmis.
cli-map-image-stale = Tallennettu projekti ei enää vastaa luotua karttaa, esimerkiksi koska alue, karttatyyppi tai mittakaava muuttui. Käynnistä ensin ajo, joka korvaa kartan ("Korvaa nykyinen kartta" sivulla tai freemapper run --force).
cli-map-image-superseded = Sillä välin pyydettiin uudempaa esikatselua, joten tätä ei laskettu.
cli-map-image-bad-content = Sisältöasetukset eivät kelpaa.
cli-map-image-bad-request = Esikatselupyyntö ei kelpaa. Lähetä JSON-objekti, tarvittaessa sisältöasetusten kanssa.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – alueen valinta
page-heading = Valitse kartta-alue
page-update-available = freemapper { $version } on saatavilla. Päivitä komennolla
page-update-notes = Julkaisutiedot
page-folder = Projektikansio
page-pick = Hae paikka
page-query-placeholder = esim. Clara-Zetkin-Schule Strausberg
page-search-button = Hae
page-searching = Haetaan …
page-no-results = Ei tuloksia.
page-point-only = vain piste, ei alue
page-draw = Piirrä itse
page-rectangle = Suorakulmio
page-polygon = Monikulmio
page-finish-polygon = Sulje monikulmio
page-clear = Tyhjennä
page-hint-rectangle = Napsauta kartalta kahta vastakkaista kulmaa.
page-hint-polygon = Napsauta kulmat peräkkäin. Sulje alue napsauttamalla ensimmäistä kulmaa, kaksoisnapsauttamalla tai painamalla "Sulje monikulmio". Esc peruu, Backspace poistaa viimeisen kulman.
page-hint-idle = Valitse paikka haulla tai piirrä alue itse.
page-summary = Alue
page-source-osm = OpenStreetMapista: { $name }
page-source-drawn = Piirretty käsin
page-size = Koko puskurin kanssa: { $width } × { $height } m
page-tiles-one = { $n } ruutu: { $names }
page-tiles-other = { $n } ruutua: { $names }
page-download = Lataus: noin { $mb } MB
page-download-none = Lataus: kaikki on jo välimuistissa
page-download-details = Mitä ladataan
page-cached = välimuistissa
page-disk = Levytila: { $volumes }
page-disk-volume = { $free } vapaana, tarvitaan noin { $needed }
page-cache-volume = välimuisti
page-project-volume = projektikansio
page-no-room = ei mahdu levylle
page-no-room-skip = Kaikki ei mahdu levylle (freemapper pitää { $reserve } vapaana). Haku jättää merkityt osat pois. Vapauta ensin tilaa, jos tarvitset ne:
page-no-room-stop = Levytilaa ei ole tarpeeksi (freemapper pitää { $reserve } vapaana). Ilman merkittyjä osia karttaa ei voi tehdä, joten haku pysähtyy ennen kuin se lataa mitään. Vapauta ensin tilaa:
page-disk-unknown = freemapper ei voi lukea vapaata levytilaa ({ $error }). Se hakee ilman tarkistusta.
page-too-large = Alue on liian suuri: enintään { $max } m sivua kohden (puskuri mukaan lukien). Valitse pienempi.
page-outside = Alue on tuettujen osavaltioiden ({ $states }) ulkopuolella; freemapperilla ei ole vielä tietolähteitä muille alueille.
page-outside-unknown = Alue on freemapperin tukemien alueiden ulkopuolella; siellä sillä ei ole vielä tietolähteitä.
page-adapters-failed = Ilmakuvia ei voi luetella ({ $detail }). Kartta näyttää vain OpenStreetMapin; kaikki muu toimii.
page-settings = Kartta
page-kind = Karttatyyppi
page-schoolyard = Koulunpihakartta (1:1000)
page-sprint = Sprinttikartta (1:4000)
page-forest = Metsäkartta (1:15000)
page-scale = Mittakaava 1:
page-map-title = Otsikko
page-contour-interval = Korkeuskäyrien väli
page-contour-auto = automaattinen
page-steep-slope = Jyrkkyysraja
page-steep-slope-auto = automaattinen (27°)
page-steep-slope-hint = Tästä kaltevuuskulmasta alkaen maasto katsotaan jyrkäksi penkereitä ja kallioita etsittäessä. 27° (automaattinen) sopii mäkimaastoon, puistoihin ja kaupunkeihin. Kalliolabyrinteissä, rotkoissa ja louhoksissa lähes jokainen rinne on jyrkkä, joten penkereet ja kalliot sulautuvat yhdeksi laikuksi eikä yhtään löydy; 55° - 65° erottaa ne toisistaan. Liian korkea arvo jättää vain jyrkimmät seinämät.
page-settings-fixed = Projekti on jo olemassa; karttatyyppi, mittakaava ja otsikko ovat tiedostossa freemapper.toml. Korkeuskäyrien väliä voi yhä muuttaa.
page-save = Tallenna
page-saving = Tallennetaan …
page-saved = Tallennettu tiedostoon freemapper.toml.
page-zone-changed = Tallennettu. Alue on nyt toisella UTM-vyöhykkeellä; freemapper hakee sen tiedot uudelleen seuraavassa ajossa.
page-provider-changed = Tallennettu. Uuden alueen korkeus- ja ilmakuvat tulevat toiselta tietojen tarjoajalta; freemapper hakee sen tiedot uudelleen seuraavassa ajossa.
page-save-stale = freemapper.toml on muuttunut toisessa ikkunassa tai toisessa ohjelmassa sen jälkeen, kun tämä sivu luki sen. Mitään ei tallennettu; sivu tietää nyt tallennetun tilan. Tallenna uudelleen korvataksesi sen tässä näytettävällä alueella.
page-has-map = Nykyinen kartta (.omap) säilyy ennallaan; freemapper tekee uudelle alueelle uudet pohjat ja ehdokkaat. Alla oleva valinta "Korvaa nykyinen kartta" (tai "freemapper run --force") tekee myös kartan uudelleen ja säilyttää vanhan tiedostona .omap.bak.
page-done = Valmis
page-source = Lähdekoodi (AGPL-3.0-or-later)
page-closed = Valmis. Voit sulkea tämän ikkunan.
page-next = Seuraavaksi: freemapper run "{ $dir }"
page-made-files = Luotu: { $files }. Kansio: { $dir }
page-back-to-agent = Kerro tekoälyavustajallesi, että alue on tallennettu.
page-nothing-saved = Mitään ei tallennettu.
page-failed = Virhe: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial-of = Ilmakuva { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = Mitä kartalle tulee
page-content-help = Profiili ratkaisee, mikä tulee suoraan kartalle ja mikä jää tarkistettavaksi ehdokkaaksi. Ryhmälle tehty valinta voittaa profiilin. Se tallennetaan alueen mukana.
page-content-profile-legend = Profiili
page-content-profile-safe = Varovainen
page-content-profile-safe-hint = Jokainen kohde pysyy siellä, minne ajo sen sijoittaa: rakennukset ja kiinteistörekisterin alueet tulevat kartalle; muurit, aidat, puut ja paljon OpenStreetMapista (lammet, polut ilman tunnettua pintaa) jäävät ehdokkaiksi.
page-content-profile-balanced = Tasapainoinen
page-content-profile-balanced-hint = Kiinteistörekisterin, rekisterien ja katukartoituksen kohteet tulevat kartalle, samoin korkeuskäyrät ja rautatien varrella olevat kielletyt alueet. Muurit, aidat, pensasaidat, penkereet, portaat, vesi, puut, muistomerkit, läpikulut, polut ja rautatiet tulevat myös OpenStreetMapista. Katukalusteet, muut kielletyt alueet ja maastomallin muodot jäävät ehdokkaiksi.
page-content-profile-full = Täysi
page-content-profile-full-hint = Kaikki tulee kartalle, kuten tarkistustulosteessa; vain piilotetut ehdokkaat jäävät pois.
page-content-groups-legend = Ryhmittäin
page-content-groups-help = "Kuten profiilissa" noudattaa profiilia. Mitä se tarkoittaa ryhmälle, riippuu siitä, mistä kukin kohde tulee.
page-content-place-profile = Kuten profiilissa
page-content-place-map = Kartta
page-content-place-candidates = Ehdokas
page-content-place-off = Pois
page-content-group-walls = Muurit
page-content-group-fences = Aidat
page-content-group-hedges = Pensasaidat
page-content-group-slopes = Penkereet ja kalliot
page-content-group-stairs = Portaat
page-content-group-trees = Puut ja puuryhmät
page-content-group-water-features = Vesi, suihkulähteet ja lähteet
page-content-group-monuments = Tornit, muistomerkit ja voimajohdot
page-content-group-furniture = Katukalusteet ja valaisimet
page-content-group-passages = Läpikulut
page-content-group-contours = Korkeuskäyrät
page-content-group-landforms = Kumpareet, painanteet, penkereet ja kalliot maastomallista
page-content-group-vegetation = Kasvillisuus laserpistepilvistä (Brandenburg, Berliini, Nordrhein-Westfalen, Rheinland-Pfalz, Saarland ja Baijeri; pois, kunnes valitset sen; lataa noin 40–600 MB neliökilometriltä)
page-content-group-out-of-bounds = Kielletyt alueet
page-content-group-paths = Polut ja rautatiet
page-content-group-streets = Kadut ja aukiot
page-run-heading = Luo kartta
page-preview-heading = Kartan esikatselu
page-preview-stored = Kuten viimeksi luotu.
page-preview-live = Tämän sivun asetuksilla, ei vielä tallennettu.
page-preview-live-saved = Tallennetuilla asetuksilla, ei vielä luotu ajossa.
page-preview-computing = Lasketaan …
page-preview-none = Karttaa ei ole vielä luotu.
page-preview-later = Tällä kartalla ei ole vielä esikatselua. Se tulee seuraavassa ajossa, joka kirjoittaa kartan.
page-preview-stale = Alue, karttatyyppi tai mittakaava on muuttunut kartan luomisen jälkeen. Kuva näyttää yhä viimeksi luodun kartan; uusi esikatselu vaatii ajon valinnalla "Korvaa nykyinen kartta".
page-preview-area-unsaved = Tämän sivun aluetta ei ole vielä tallennettu. Kuva näyttää viimeksi luodun kartan; uusi alue vaatii ajon valinnalla "Korvaa nykyinen kartta".
page-preview-run = Kuten viimeksi luotu. Sivu päivittää kuvan, kun ajo päättyy.
page-preview-open = Avaa täysikokoisena
page-preview-alt = Karttakohteiden esikatselu
page-opt-ocad = Kirjoita OCAD-aloitusprojekti
page-help-ocad = Kaikille, jotka jatkavat OCADissa.
page-opt-pdf = Tulostettava PDF
page-help-pdf = Vain karttakohteet, ei ehdokkaita.
page-opt-pdf-candidates = Tarkistustuloste kaikkine ehdokkaineen
page-opt-training = Harjoituskartta
page-help-training = Kartta ja kaikki ehdokkaat siivottuna tulostettavaksi sellaisenaan; OCADin kanssa myös OCAD-tiedostona.
page-views-heading = Harjoituskartan näkymät
page-view-no-paths = Ilman polkuja (Ultimate-kartta)
page-view-brown-only = Vain ruskea (korkeuskäyräsuunnistus)
page-view-no-north-lines = Ilman pohjoisviivoja
page-view-custom = Toinen näkymä
page-help-views = Jokainen valittu näkymä on yksi lisätiedosto harjoituskartan viereen, `<name>-training-<view>.omap` ja sen PDF, OCADin kanssa myös OCAD-tiedosto. Toinen näkymä on `no:505,506` (jätä symbolit pois) tai `only:101,102` (säilytä vain nämä); `+` yhdistää näkymiä. Otsikko, mittakaava ja lähteet säilyvät aina.
page-help-views-forest = Ilman polkuja ja vain ruskea ovat vain metsäkartoille.
page-opt-scale-bar = Mittakaavajana PDF-tiedostoihin
page-help-scale-bar = Saksalaisissa kartoissa sitä ei yleensä ole.
page-opt-sources = Lähderivit PDF-tiedostoihin
page-sources-auto = automaattinen
page-sources-full = täysi
page-sources-credit-only = vain freemapper
page-warn-credit-only = PDF-tiedostoissa mainitaan silloin vain freemapper. Tietolisenssit (ODbL, dl-de/by-2-0) edellyttävät lähteen mainitsemista, kun luovutat kartan eteenpäin; silloin mainitset lähteet itse. ATTRIBUTION.txt luettelee ne kaikki.
page-need-pdf = Mittakaavajana ja lähderivit koskevat vain PDF:n luontia.
page-opt-offline = Vain välimuisti, ei verkkoa
page-opt-force = Korvaa nykyiset kartat
page-help-force = Kartta ja, jos "Harjoituskartta" on valittu, myös harjoituskartta. Vanhat tiedostot säilyvät tiedostoina .omap.bak.
page-opt-permits = Lupahuomautukset
page-help-permits = Hallinnolliset yksiköt, suojelualueet ja keneltä kysyä, tiedostossa permits.md. Vihje, ei oikeudellista neuvontaa.
page-run-start = Luo kartta
page-run-again = Luo uudelleen
page-run-cancel = Peruuta
page-run-cancelling = Peruutetaan …
page-warnings-heading = Varoitukset
page-where-files = Tiedostot ovat projektikansiossa:
page-open-folder = Avaa kansio
page-log-label = Ajon loki
page-force-title = Korvataanko nykyiset kartat?
page-force-text = Nykyinen kartta korvataan, ja jos "Harjoituskartta" on valittu, myös harjoituskartta. Muokkauksesi eivät katoa: vanhat tiedostot jäävät projektikansioon tiedostoina .omap.bak. Jatketaanko?
page-force-cancel = Peruuta
page-force-confirm = Korvaa
page-step-fetch = Haetaan tietoja
page-step-derive = Lasketaan pohjia
page-step-map = Kirjoitetaan karttaa
page-step-ocad = Kirjoitetaan OCAD-projektia
page-step-pdf = Kirjoitetaan PDF:ää
page-step-pdf-candidates = Kirjoitetaan tarkistustulostetta
page-step-training = Kirjoitetaan harjoituskarttaa
page-step-permits = Kirjoitetaan lupahuomautuksia
page-step-line = Vaihe { $index } / { $total }: { $label }
page-starting = Ajo käynnistyy …
page-file-kind-map = kartta
page-file-kind-template = pohja
page-file-kind-ocad = OCAD-kartta
page-file-kind-ocad-candidates = OCAD-ehdokkaat
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = tarkistustuloste
page-file-kind-training-map = harjoituskartta
page-file-kind-training-pdf = harjoituskartan PDF
page-file-kind-training-ocad = OCAD-harjoituskartta
page-file-kind-training-view-map = harjoituskartan näkymä
page-file-kind-training-view-pdf = harjoituskartan näkymän PDF
page-file-kind-training-view-ocad = OCAD-harjoituskartan näkymä
page-file-kind-preview = kartan esikatselu
page-file-kind-permits = lupahuomautukset
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Valmis. Tiedostot on kirjoitettu.
page-result-failed = Ajo epäonnistui: { $error }
page-result-failed-kept = Kirjoitettu säilyy projektikansiossa; uusi ajo jatkaa siitä.
page-result-cancelled = Peruutettu. Jo ladattu säilyy välimuistissa; uusi ajo jatkaa siitä.
page-result-title-done = Kartta luotu
page-result-title-failed = Ajo epäonnistui
page-result-title-cancelled = Ajo peruutettu
page-kept-map = Nykyinen kartta jätettiin ennalleen. "Korvaa nykyinen kartta" tekee sen uudelleen ja säilyttää vanhan tiedostona .omap.bak.
page-no-files = Tiedostoja ei luotu.
page-opened-folder = Kansio on auki.
page-open-failed = { $message } Kansio: { $dir }
page-why-no-area = Valitse ensin alue.
page-why-checking = Esikatselua lasketaan …
page-why-too-large = Alue on liian suuri.
page-why-outside = Alue on tuettujen osavaltioiden ulkopuolella.
page-why-disk = Levytilaa ei ole tarpeeksi (katso yllä).
page-why-running = Ajo on käynnissä.
page-err-scale-bar-no-pdf = Mittakaavajana vaatii PDF:n.
page-err-sources-no-pdf = Lähderivit vaativat PDF:n.
page-save-failed = Aluetta ei tallennettu; ajoa ei käynnistetty.
page-poll-lost = Palvelin ei vastaa. freemapper yrittää edelleen.
page-run-unknown = Palvelin ei enää tunne tätä ajoa.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Sulje

## update (spec 3.4 to 3.6)

update-source-build = Tämä freemapper on rakennettu lähdekoodista. Päivitä se komennoilla git pull ja cargo build --release --locked.
update-current = freemapper { $version } on ajan tasalla.
update-available = freemapper { $new } on saatavilla (sinulla on { $old }), julkaistu { $released }.
update-details =
    Julkaisutiedot: { $notes }
    Lataus: { $size } MB
update-decimal-separator = ,
update-ask = Asennetaanko se nyt? [y/N]
update-not-installed = Mitään ei asennettu.
update-needs-yes = syöte ei ole pääte, joten freemapper ei voi kysyä ennen asennusta; asenna kysymättä komennolla freemapper update --yes
update-installing = ladataan ja tarkistetaan freemapper { $version }
update-done = freemapper päivitettiin versiosta { $old } versioon { $new }. Julkaisutiedot: { $notes }
update-mcp-restart = Käynnissä oleva MCP-palvelin pitää vanhan version, kunnes tekoälyavustaja käynnistää sen uudelleen.
update-rolled-back = Palattiin versioon freemapper { $version }. Jätetty versio säilyy tiedostona { $path }.
update-not-writable = ohjelmakansioon { $path } ei voi kirjoittaa, eikä freemapper pyydä järjestelmänvalvojan oikeuksia. Pura uusi julkaisu sinne käsin tai siirrä freemapper kansioon, johon voit kirjoittaa.
update-too-old = freemapper { $new } ei asennu version { $old } itsepäivityksellä; asenna { $needed } tai uudempi käsin, katso { $notes }
update-busy = toinen päivitys tai palautus on käynnissä ohjelmakansiossa (lukkotiedosto { $path }); odota sen päättymistä ja yritä uudelleen
update-bad-signature = julkaisua ei voitu vahvistaa ({ $error }); mitään ei asennettu
update-check-bad-signature = uusinta freemapper-julkaisua ei voitu vahvistaa ({ $error }); mitään ei asennettu
update-no-manifest = uusin julkaisu ei voi asentaa itseään (siinä ei ole päivitysmanifestia); lataa se osoitteesta { $page }
update-no-manifest-version = julkaisua { $version } ei ole tai se ei voi asentaa itseään (siinä ei ole päivitysmanifestia); katso { $page }
update-no-asset = julkaisussa ei ole ohjelmaa tälle järjestelmälle ({ $platform }); mitään ei asennettu
update-no-previous = ei ole edellistä versiota, johon palata ({ $path } puuttuu)
update-failed = päivitys epäonnistui: { $error }
update-mode-set = Päivitystilaksi asetettu { $mode } tiedostossa { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } ohittaa tämän asetuksen, kun se on asetettu.
update-mode-failed = tiedostoa { $path } ei voitu kirjoittaa: { $error }
update-notice = freemapper { $new } on saatavilla (sinulla on { $old }). Päivitä komennolla: freemapper update
update-applies-next-start = Uusi versio tulee käyttöön freemapperin seuraavassa käynnistyksessä.
update-install-failed = freemapper { $new } ei asentunut: { $error }
update-no-config-dir = tässä järjestelmässä ei ole asetuskansiota; aseta FREEMAPPER_CONFIG_DIR
mcp-adapter-note = Huomautus henkilölle: käynnistyessään freemapper-palvelin havaitsi sovitinkansioista tämän. Välitä se eteenpäin; henkilö korjaa kansion tai poistaa sen ja käynnistää palvelimen sitten uudelleen.
    { $lines }
update-mcp-note = Huomautus henkilölle: freemapper { $new } on saatavilla (tämä on { $old }). Kysy häneltä ennen kuin kutsut install_update.
update-mcp-needs-confirm = mitään ei asennettu; kysy ensin henkilöltä ja kutsu install_update arvolla confirm: true, kun hän suostuu
update-mcp-note-manual = Huomautus henkilölle: freemapper { $new } on saatavilla (tämä on { $old }), mutta install_update ei voi asentaa sitä; asenna { $needed } tai uudempi käsin, katso { $notes }
update-mcp-note-bad-signature = Huomautus henkilölle: uusinta freemapper-julkaisua ei voitu vahvistaa ({ $error }); mitään ei asennettu. Kerro hänelle äläkä asenna sitä muulla tavalla.
update-mcp-restart-now = Käynnistä freemapperin MCP-palvelin nyt uudelleen; se käyttää yhä vanhaa versiota.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = virhe
error-cancelled = peruutettu
cli-cancelled-kept = peruutettu; säilytetty: { $files }
error-io = { $path }: { $message }
error-io-disk-full = levy on täynnä, joten freemapper ei voinut kirjoittaa { $path }. Vapauta tilaa ja yritä uudelleen

## Errors: project

error-project-already-exists = projekti on jo olemassa: { $path }
error-project-map-exists = kartta on jo olemassa (siinä voi olla työtäsi); korvaa se valitsimella --force: { $path }
error-project-invalid-scale = virheellinen mittakaava 1:{ $scale }
error-project-invalid-contour-interval = virheellinen korkeuskäyrien väli { $interval } m tiedostossa freemapper.toml; sen on oltava suurempi kuin 0
error-project-invalid-steep-slope = virheellinen jyrkkyysraja { $slope }° tiedostossa freemapper.toml; sen on oltava välillä 15° ja 80°
error-project-scale-not-allowed = mittakaava 1:{ $scale } ei ole sallittu karttatyypille { $kind }; käytä { $allowed }
error-project-scale-not-allowed-enlarge = mittakaava 1:{ $scale } ei ole sallittu karttatyypille { $kind }; käytä { $allowed }. Piirrä mittakaavassa 1:4000, freemapper-print suurentaa tulostettaessa mittakaavaan 1:3000
word-kind-schoolyard = koulunpiha
word-kind-sprint = sprintti
word-kind-forest = metsä
error-project-toml = virheellinen projektitiedosto: { $message }
error-project-unknown-adapter = projektitiedosto nimeää sovittimen "{ $adapter }", jota tällä freemapperilla ei ole; uudempi freemapper voi tuoda sen (`freemapper update`), tai se tulee sovitinkansiostasi (`freemapper adapters list` näyttää sen) tai, komentorivin valitsimella --adapters-from-project, projektin kansiosta `adapters` (vain projektille, johon luotat)
error-project-not-fetched = ruutuja ei ole vielä haettu; aja ensin `freemapper fetch`
error-project-ocd = OCAD-vienti: { $message }
error-project-pdf = PDF: { $message }
error-project-template-outdated = { $path }: kuva ja world-tiedosto eivät vastaa kartta-aluetta; aja `freemapper derive` uudelleen
error-project-tiles-outdated = kartta-alue tarvitsee ruudut { $tiles }, joita ei haettu; aja `freemapper fetch`
error-project-buildings-not-fetched = tälle kartta-alueelle ei ole tallennettu ALKIS-rakennuksia; aja ensin `freemapper fetch` verkossa
error-project-orthophoto-not-fetched = tälle kartta-alueelle ei ole tallennettu ortokuvaa; aja ensin `freemapper fetch` verkossa
error-project-terrain-not-fetched = tälle kartta-alueelle ei ole tallennettu maastomallia; aja ensin `freemapper fetch` verkossa
error-project-candidates-outdated = tallennetut OSM-ehdokkaat eivät kata kartta-aluetta; aja `freemapper fetch`
error-project-stored = { $path }: { $message }; aja `freemapper fetch` uudelleen
error-project-area-too-large = kartta-alue puskureineen on { $width } × { $height } m, yli { $limit } m sivua kohden; pienennä kartta-aluetta tai arvoa buffer_m tiedostossa freemapper.toml
error-project-area-crosses-country = kartta-alue on maan rajan molemmin puolin; rajan yli ulottuvia karttoja ei vielä tueta, joten piirrä alue yhden maan sisään
error-project-permits-not-for-region = lupahuomautukset eivät ole vielä saatavilla tälle alueelle: ne kysyvät saksalaisilta palveluilta ja nimeävät saksalaisia viranomaisia
error-project-outside-coverage = kartta-alue on tuettujen osavaltioiden ({ $states }) ulkopuolella; freemapperilla on toistaiseksi tietolähteet vain niille

## Errors: disk space

error-space-required = levytilaa ei ole tarpeeksi kohteelle { $part }. Se tarvitsee { $needed }, mutta { $path } on vapaana vain { $left } { $reserve } varauksen jälkeen. Vapauta tilaa ja aja `freemapper fetch` uudelleen
error-space-skipped = freemapper ohittaa kohteen { $part } levytilan puutteen vuoksi. Se tarvitsee { $needed }, mutta { $path } on vapaana vain { $left } { $reserve } varauksen jälkeen. Vapauta tilaa ja aja `freemapper fetch` uudelleen
word-part-map-data = ALKIS- ja OSM-karttatiedot
word-part-terrain = maastomalli
word-part-orthophoto = ortokuva
word-part-surface = nDSM-pohjan pintamalli
word-part-osm-update = välimuistissa olevan OSM-otteen päivitys
word-part-k-five = Berliinin K5-karttapohja
word-part-berlin-layers = Berliinin ALKIS-rakenteet, puurekisteri, katukartoitus ja pistetasot
word-part-state-layers = täydet ALKIS-rakenteet, maastomallin kohteet ja puurekisterit
word-part-point-cloud = kasvillisuusehdokkaiden laserpistepilvet

## Errors: downloads

error-fetch-http = osoitteen { $url } lataus epäonnistui: { $message }
error-fetch-http-temporary = osoitteen { $url } lataus epäonnistui: { $message }. Palvelu on todennäköisesti ylikuormitettu tai huollossa; freemapper yritti enintään { $attempts } kertaa. Käynnistä ajo muutaman minuutin kuluttua uudelleen; jo ladattu säilyy.
error-fetch-corrupt-archive = osoitteen { $url } lataus ei ole täydellinen ruutuarkisto: { $message }
error-fetch-corrupt-extract = osoitteen { $url } lataus ei ole täydellinen OSM-ote: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] julkaistut ruudut eivät kata ruutua { $tiles }; pienennä kartta-aluetta tai arvoa buffer_m tiedostossa freemapper.toml
       *[other] julkaistut ruudut eivät kata ruutuja { $tiles }; pienennä kartta-aluetta tai arvoa buffer_m tiedostossa freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] ei verkossa eikä välimuistissa: { $product }-ruutu { $tiles }; aja `freemapper fetch` verkossa
       *[other] ei verkossa eikä välimuistissa: { $product }-ruudut { $tiles }; aja `freemapper fetch` verkossa
    }
word-product-orthophoto = ortokuva (DOP20)
word-product-dtm = maastomalli (DGM1)
word-product-image-dsm = pintamalli (bDOM)
word-product-saxony-dtm = maastomalli (GeoSN DGM1)
word-product-saxony-dsm = pintamalli (GeoSN DOM1)
error-fetch-osm-extract-missing = OSM-otetta ei ole välimuistissa; aja `freemapper fetch` verkossa
error-fetch-not-found = { $name } ei löydy palvelimelta
error-fetch-refused = { $url }: palvelin hylkäsi pyynnön (HTTP { $status })
error-fetch-refused-detail = { $url }: palvelin hylkäsi pyynnön (HTTP { $status }): { $detail }
error-fetch-too-large = osoitteen { $url } lataus on suurempi kuin { $limit } MB; pysäytetty
error-fetch-not-stored = tälle kartta-alueelle ei ole tallennettu: { $label }: { $path } puuttuu, sitä ei voi lukea tai se ei kata aluetta; aja ensin `freemapper fetch` verkossa
error-fetch-range-ignored = { $url }: palvelin ohitti tavualueen { $range } (HTTP { $status })
error-fetch-not-allowed = { $url }: freemapper ei lähetä pyyntöjä tähän osoitteeseen: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper ei lähetä pyyntöjä tähän osoitteeseen: kohteen { $name } sovitin ei nimeä palvelinta { $host }{ $from }. Jos palvelu on siirtynyt sinne, salli palvelin komennolla `freemapper adapters allow-host { $adapter } { $host }` ja ilmoita siitä osoitteessa https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper ei lähetä pyyntöjä tähän osoitteeseen: sovitin { $adapter } ei nimeä palvelinta { $host }{ $from }. Jos palvelu on siirtynyt sinne, lisää palvelin sovittimen manifestin kohtaan `[net] hosts`
error-fetch-host-redirected-from = {" "}({ $url } ohjaa sinne)
error-reason-not-a-host-name = ei ole palvelimen nimi
error-reason-not-ascii = ASCII-merkistön ulkopuolinen nimi on kirjoitettava xn--muodossa
error-reason-not-a-dns-name = ei ole DNS-nimi
error-reason-no-domain = tämän tietokoneen nimi tai nimi ilman verkkotunnusta
error-reason-local-name = paikallisverkon nimi
error-reason-ip-address = IP-osoite, ei DNS-nimi
error-reason-unexpected-character = odottamaton merkki
error-reason-not-https = ei https
error-reason-user-info = käyttäjätiedot
error-reason-port = muu portti kuin 443
error-reason-host-not-declared = palvelin ei ole ilmoitettujen palvelimien joukossa
error-reason-local-address = tämän tietokoneen tai paikallisverkon osoite, ei internetin palvelimen
error-reason-too-many-redirects = yli { $count } uudelleenohjausta
error-reason-redirect-nowhere = uudelleenohjaus (HTTP { $status }) ei minnekään
error-fetch-currency-csv = ajantasaisuus-CSV, rivi { $line }: { $message }
error-alkis = virheelliset ALKIS-tiedot: { $message }
error-osm = OSM-tiedot: { $message }

## Errors: templates and rasters

error-derive-zip = ruutuarkisto: { $message }
error-derive-image = kuva: { $message }
error-derive-no-tiles = ei ruutuja, joista laskea
error-derive-area-too-large = kartta-alue on liian suuri yhdelle pohjalle: { $width } × { $height } pikseliä (enintään { $limit } sivua kohden); pienennä kartta-aluetta tai arvoa buffer_m tiedostossa freemapper.toml
error-raster-world-file = virheellinen world-tiedosto: { $message }
error-raster-misaligned = rasterit eivät ole samassa pikseliruudukossa: { $message }
error-raster-pixel-size = rastereilla on eri pikselikoot: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = virheellinen GeoJSON: { $message }
error-area-no-polygon = GeoJSON ei sisällä monikulmiota
error-area-multi-polygon = MultiPolygon, jossa on { $parts } osaa; jaa kartta-alue yhdeksi monikulmioksi
error-area-out-of-range = koordinaatti arvoalueen ulkopuolella: lat { $lat }, lon { $lon }
error-area-not-an-area = ei kelvollinen kartta-alue: { $message }
error-georef-date = päivämäärä { $date } on World Magnetic Modelin voimassaoloajan ulkopuolella
error-georef-map-range = piste E { $easting } N { $northing } on kartan koordinaattialueen ulkopuolella
error-omap-xml = virheellinen OOM-XML: { $message }
error-omap-missing-element = elementtiä <{ $name }> ei löydy
error-ocd-duplicate-symbol = symbolinumeroa { $symbol } käytetään kahdesti
error-ocd-unknown-symbol = kohde viittaa symboliin { $symbol }, jota ei ole tiedostossa
error-ocd-wrong-type = symbolin { $symbol } kohteella on symbolilleen väärä tyyppi

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Käyttö:
cli-heading-commands = Komennot:
cli-heading-options = Valitsimet:
cli-heading-arguments = Argumentit:
cli-help-possible-values = mahdolliset arvot
cli-help-default = oletus
cli-arg-help = Näytä ohje
cli-arg-version = Näytä versio
cli-help-help = Näytä komennon ohje
cli-arg-help-command = Komento, jonka ohje näytetään, esimerkiksi run
cli-parse-hint = Lisätietoja saat valitsimella '--help'.
cli-double-click-start = freemapper avaa selaimessa sivun, jolla valitset kartta-alueen. Projektikansio on { $path }. Sulje tämä ikkuna lopettaaksesi ohjelman.
cli-parse-unknown-argument = odottamaton argumentti { $arg }
cli-parse-missing-required = pakollinen argumentti puuttuu: { $args }
cli-parse-invalid-value = virheellinen arvo { $value } kohteelle { $arg }
cli-parse-invalid-value-choices = virheellinen arvo { $value } kohteelle { $arg }; mahdolliset arvot: { $values }
cli-parse-invalid-subcommand = tuntematon komento { $value }
cli-parse-missing-subcommand = komento vaaditaan
cli-parse-too-many-values = odottamaton arvo { $value } kohteelle { $arg }
cli-parse-argument-conflict = { $arg } ei voi käyttää yhdessä valitsimen { $other } kanssa

help-freemapper = Rakentaa OOM-aloitusprojekteja paikkatiedoista
help-freemapper-lang = Ohjelmatekstien kieli, `de` tai `en` (oletus: FREEMAPPER_LANG, sitten järjestelmän kieli, sitten englanti)
help-init = Luo projektikansio kartta-alueelle
help-init-dir = Luotava projektikansio
help-init-geojson = GeoJSON-tiedosto, jossa on kartta-alueen monikulmio (EPSG:4326)
help-init-osm = Kartta-alueen OSM-kohde, esim. way/992423119
help-init-place = Paikan nimi, esim. "Clara-Zetkin-Schule Strausberg"; freemapper käyttää parasta aluetta vastaavaa osumaa
help-init-part = Mitä osaa käytetään, kun OSM-alue koostuu useista erillisistä osista (1 on suurin)
help-init-kind = Karttatyyppi
help-arg-scale = Uuden projektin mittakaavan nimittäjä: koulunpiha 1000 (oletus); sprintti 4000 (oletus) tai 5000; metsä 15000 (oletus), 10000 tai 7500. Muut arvot hylätään, koska freemapper-print hylkää ne
help-init-title = Kartan otsikko reunatietoja varten (oletus: kansion nimi)
help-export = Kirjoita aloitusprojekti (OOM-kartta, pohjat, lähdemerkinnät) projektikansioon
help-export-dir = Projektikansio
help-arg-force = Korvaa olemassa oleva karttatiedosto ja säilytä vanha tiedostona `<name>.omap.bak`
help-export-ocad = Kirjoita myös OCAD-aloituspiste kansioon `ocad/` (OCAD 12 -muoto); olemassa oleva OOM-kartta säilyy, ellet anna valitsinta --force
help-export-pdf = Kirjoita myös `<name>.pdf`, tulostettava kartta (vain karttakohteet, ei ehdokkaita); olemassa oleva OOM-kartta säilyy, ellet anna valitsinta --force
help-export-pdf-candidates = Kirjoita myös `<name>-candidates.pdf`, tarkistustuloste: kartta kaikkine ehdokkaineen, paikkatarkasti; olemassa oleva OOM-kartta säilyy, ellet anna valitsinta --force
help-export-training = Kirjoita myös harjoituskartta `<name>-training.omap` ja sen PDF `<name>-training.pdf`: kartta ja kaikki ehdokkaat siivottuna tulostettavaksi sellaisenaan; valitsimella --ocad myös `ocad/<name>-training.ocd`. Olemassa oleva OOM-kartta säilyy, ellet anna valitsinta --force, ja --force korvaa myös olemassa olevan harjoituskartan
help-export-view = Valitsimen --training kanssa: kirjoita myös harjoituskartta ilman sitä, minkä näkymä jättää pois, tiedostoina `<name>-training-<view>.omap` ja sen PDF (valitsimella --ocad myös OCAD-tiedosto). Näkymät: `no-paths` (ei teitä, polkuja eikä portaita), `brown-only` (vain ruskeat symbolit ja kehys), `no-north-lines`, tai luettelo `no:<codes>` / `only:<codes>`; yhdistä merkillä `+`. Toista valitsin usealle tiedostolle
help-export-pdf-scale-bar = Piirrä mittakaavajana PDF-tiedostoihin (oletuksena pois, saksalaisissa kartoissa sitä ei yleensä ole; mittakaavaluku jää otsikkolohkoon)
help-export-pdf-sources = Lähderivit PDF-tiedostoissa: `auto` tulostaa koko luettelon ja lyhentää sen vain, jos se veisi yli 5 riviä, `full` ei lyhennä koskaan, `credit-only` tulostaa vain freemapperin lähderivin (ODbL ja dl-de/by-2-0 edellyttävät silloin lähteiden mainitsemista, kun luovutat kartan eteenpäin). ATTRIBUTION.txt luettelee aina kaikki lähteet
help-fetch = Lataa kartta-alueen tiedot (ortokuva, maastomalli, nDSM, ALKIS-rakennukset, OSM)
help-fetch-dir = Projektikansio
help-fetch-offline = Käytä vain välimuistia, ei verkkoyhteyttä
help-arg-cache-dir = Ladattujen ruutujen välimuistikansio (oletus: järjestelmän välimuistikansio + `freemapper`)
help-derive = Laske pohjat (ortokuva, rinnevarjostus, kaltevuus, paikallinen reliefi, nDSM), korkeuskäyrät ja maastonmuodot haetuista tiedoista
help-derive-dir = Projektikansio
help-run = Hae, laske ja vie yhdellä kertaa
help-run-dir = Projektikansio
help-run-offline = Käytä vain välimuistia, ei verkkoyhteyttä
help-run-ocad = Kirjoita myös OCAD-aloituspiste kansioon `ocad/` (OCAD 12 -muoto)
help-run-pdf = Kirjoita myös `<name>.pdf`, tulostettava kartta (vain karttakohteet, ei ehdokkaita)
help-run-pdf-candidates = Kirjoita myös `<name>-candidates.pdf`, tarkistustuloste: kartta kaikkine ehdokkaineen, paikkatarkasti
help-run-training = Kirjoita myös harjoituskartta `<name>-training.omap` ja sen PDF `<name>-training.pdf`: kartta ja kaikki ehdokkaat siivottuna tulostettavaksi sellaisenaan; valitsimella --ocad myös `ocad/<name>-training.ocd`. Olemassa oleva harjoituskartta säilyy, ellet anna valitsinta --force
help-run-view = Valitsimen --training kanssa: kirjoita myös harjoituskartta ilman sitä, minkä näkymä jättää pois, tiedostoina `<name>-training-<view>.omap` ja sen PDF (valitsimella --ocad myös OCAD-tiedosto). Näkymät: `no-paths` (ei teitä, polkuja eikä portaita), `brown-only` (vain ruskeat symbolit ja kehys), `no-north-lines`, tai luettelo `no:<codes>` / `only:<codes>`; yhdistä merkillä `+`. Olemassa oleva näkymätiedosto säilyy, ellet anna valitsinta --force
help-run-pdf-scale-bar = Piirrä mittakaavajana PDF-tiedostoihin (oletuksena pois; mittakaavaluku jää otsikkolohkoon)
help-run-pdf-sources = Lähderivit PDF-tiedostoissa: `auto` tulostaa koko luettelon ja lyhentää sen vain, jos se veisi yli 5 riviä, `full` ei lyhennä koskaan, `credit-only` tulostaa vain freemapperin lähderivin (ODbL ja dl-de/by-2-0 edellyttävät silloin lähteiden mainitsemista, kun luovutat kartan eteenpäin). ATTRIBUTION.txt luettelee aina kaikki lähteet
help-select = Valitse kartta-alue selaimessa piirtämällä se tai valitsemalla paikka. Tallennus kirjoittaa tiedoston freemapper.toml ja luo tarvittaessa projektikansion
help-select-dir = Projektikansio (luodaan tallennettaessa, jos sitä ei ole)
help-select-kind = Uuden projektin karttatyyppi
help-select-title = Uuden projektin kartan otsikko
help-select-no-browser = Tulosta vain sivun osoite; älä avaa selainta
help-select-port = Portti osoitteessa 127.0.0.1 (oletus: vapaa portti)
help-select-offline = Sivu käynnistyy valinnalla "Vain välimuisti, ei verkkoa"
help-select-ocad = Sivu käynnistyy valinnalla OCAD-aloitusprojekti
help-select-pdf = Sivu käynnistyy valinnalla tulostettava PDF
help-select-pdf-candidates = Sivu käynnistyy valinnalla tarkistustuloste (kaikki ehdokkaat)
help-select-training = Sivu käynnistyy valinnalla harjoituskartta
help-select-pdf-scale-bar = Sivu käynnistyy valinnalla mittakaavajana PDF-tiedostoihin
help-select-pdf-sources = Sivu käynnistyy tällä lähderivien valinnalla PDF-tiedostoissa, kuten komennossa `run`
help-find = Luettele OpenStreetMapin paikat nimelle, alueet ensin, tunnuksella komentoa init --osm varten
help-find-query = Paikan nimi, esim. "Clara-Zetkin-Schule Strausberg"
help-find-json = Tulosta osumat JSON-muodossa, samat kentät kuin MCP-työkalussa find_place
help-mcp = Tarjoa työkalut tekoälyagentille MCP:n kautta (stdin/stdout)
help-update = Asenna uudempi freemapper GitHubin julkaisuista kysyttyäsi. Lähdekoodista rakennettu versio kertoo sen sijaan, miten se päivitetään
help-update-check = Vain tarkista: poistumiskoodi 10, kun uudempi versio on olemassa, 0, kun tämä on ajan tasalla
help-update-yes = Asenna kysymättä; tarvitaan, kun syöte ei ole pääte
help-update-version = Asenna tämä versio (x.y.z) uusimman sijaan, myös vanhempi
help-update-rollback = Palaa versioon, jonka viimeisin päivitys korvasi
help-update-mode = Tallenna, mitä päivittäinen tarkistus tekee löytäessään uudemman version (oletus: notify). Toimii jokaisessa versiossa, koska se vain kirjoittaa asetuksen
help-adapters = Näytä aluesovittimet: osavaltiot ja maat, joille freemapperilla on tietolähteitä
help-adapters-list = Luettele sovittimet sekä mistä kukin tulee (sisäänrakennettu, oma kansio, projekti), sen versio, viimeisin tarkistuspäivä ja palvelimet, joita se kysyy
help-adapters-list-json = Tulosta luettelo JSON-muodossa
help-adapters-list-project = Lue myös tämän projektikansion kansion `adapters` sovittimet, kuten --adapters-from-project tekisi
help-adapters-check = Tarkista sovittimen kansio ennen käyttöä tai eteenpäin antamista: latautuisiko se, mitä sen tekijän pitäisi tietää, ja valitsimella --live vastaavatko sen palvelut. Poistumiskoodi 0: läpäisee; 1: sisältää havaintoja; 2: tarkistusta ei voitu ajaa
help-adapters-check-dir = Sovittimen kansio; sen nimi on sovittimen tunnus
help-adapters-check-live = Kysy myös sovittimen palveluilta valitsimen --bbox nimeämää suorakulmiota ja kerro, mitä kukin tuottaa. Pyynnöt pysyvät manifestin ilmoittamilla palvelimilla
help-adapters-check-bbox = Live-tarkistuksen suorakulmio asteina, muodossa minLon,minLat,maxLon,maxLat. Sen on oltava sovittimen rajojen sisällä; 150 m × 150 m riittää
help-adapters-check-json = Tulosta tulos JSON-muodossa
help-adapters-probe = Kysy jokaiselta sisäänrakennettujen adapterien mainitsemalta palvelulta kerran ja kerro, vastaako se: ok, slow, moved tai error. Poistumiskoodi on 0 myös silloin, kun palvelu epäonnistuu; tulos on vastaus
help-adapters-probe-json = Tulosta tulos JSON-muodossa
help-adapters-probe-status = Lue viime yön tila tästä tiedostosta (se saa puuttua) ja tulosta uusi tila JSON-muodossa: palvelu on poissa käytöstä kahden huonon yön jälkeen peräkkäin. Muutokset menevät vakiovirheeseen
help-adapters-allow-host = Salli sisäänrakennetulle sovittimelle yksi palvelin lisää: osavaltiolle, jonka palvelu on siirtynyt palvelimelle, jota sen sovitin ei vielä nimeä. Hylättyä osoitetta koskeva viesti kertoo palvelimen. Palvelin tarkistetaan kuten manifestin palvelin, ja freemapper yhdistää edelleen vain internetin palvelimiin
help-adapters-allow-host-adapter = Sisäänrakennetun sovittimen tunnus, kuten `adapters list` sen näyttää
help-adapters-allow-host-host = Sallittava palvelin: DNS-nimi, ei osoite, ei porttia
help-adapters-disallow-host = Peru palvelin, jonka allow-host salli
help-adapters-disallow-host-adapter = Sisäänrakennetun sovittimen tunnus
help-adapters-disallow-host-host = Peruttava palvelin
help-adapters-export = Kirjoittaa osavaltion sisäänrakennetun sovittimen muutettavaksi sovitinkansioksi: sen manifesti ja manifestin mainitsemat tiedostot kansioon `<dir>/<id>`. Kansiota ei saa olla vielä olemassa tai sen on oltava tyhjä. Muuta osoitteet sen tiedostossa `manifest.toml`, tarkista se komennolla `adapters check` ja laita se omaan sovitinkansioosi: se korvaa silloin sisäänrakennetun sovittimen varoituksen kera. Kansion poistaminen palauttaa sisäänrakennetun sovittimen
help-adapters-export-adapter = Sisäänrakennetun sovittimen tunniste sellaisena kuin `adapters list` sen näyttää
help-adapters-export-dir = Kansio, johon kansio `<id>` luodaan
help-arg-adapters-from-project = Lataa myös projektikansion kansion `adapters` sovittimet. freemapper nimeää jokaisen niistä palvelimineen, joita se kysyy. Käytä projektille, johon luotat: sovitin saa freemapperin kysymään sen nimeämiä palvelimia

## The summary of a run, for the MCP server and the area page

cli-internal-error = sisäinen virhe: { $message }
cli-pdf-sources-invalid = pdf_sources on oltava auto, full tai credit-only, ei { $value }
cli-view-unknown = näkymä on no-paths, brown-only, no-north-lines, no:<codes> tai only:<codes>, ei { $value }
cli-view-empty = luettelo { $value } ei nimeä yhtään symbolikoodia
cli-view-bad-code = { $value } ei ole symbolikoodi, kuten 505 tai 501.6
cli-summary-training-view = Kirjoitettu { $map } ja { $pdf } (harjoituskartan näkymä)
cli-summary-kept-training-view = Säilytetty harjoituskartan näkymän { $view } tiedostot; anna --force uusiaksesi ne
cli-summary-training-view-failed = Harjoituskartan näkymä epäonnistui: { $error }
cli-estimate-disk-unreadable = kohteen { $path } vapaata levytilaa ei voi lukea ({ $message })
cli-summary-wrote-map = Kirjoitettu kartta { $path }
cli-summary-kept-map = Säilytetty nykyinen kartta { $path } (siinä voi olla kartoittajan työtä)
cli-summary-map-lacks-templates = Kartta ei viittaa vielä kuvapohjiin. Käynnistä putki valinnalla force korvataksesi sen; vanha säilyy tiedostona .omap.bak.
cli-summary-map-lacks-buildings = Kartalla ei ole vielä ALKIS-rakennuksia. Käynnistä putki valinnalla force lisätäksesi ne; se korvaa kartan ja säilyttää vanhan tiedostona .omap.bak.
cli-summary-map-lacks-canopies = Kartalta puuttuvat ALKIS-katokset ja autokatokset (522), jotka ovat nyt karttakohteita. Käynnistä putki valinnalla force lisätäksesi ne; se korvaa kartan ja säilyttää vanhan tiedostona .omap.bak.
cli-summary-map-light = Pohjakartta: { $joined } viivaa yhdistetty, { $left } piilotettua tai liian pientä jätetty ehdokastasoille (mitään ei siirretty)
cli-summary-map-light-kept = Ehdokastasoissa on nyt myös { $left } kohdetta tai viivan osaa, jotka tämä versio ottaa pois pohjakartalta (merkitty freemapper:generalized). Säilytetyllä kartalla ne ovat yhä, joten ne näkyvät kahdesti. Käynnistä putki valinnalla force siivotaksesi kartan; se säilyttää vanhan tiedostona .omap.bak.
cli-summary-map-old-candidates = Kartta viittaa yhä tiedostoon templates/candidates.omap; tämä ajo täytti sen kaikilla ehdokkailla. Ehdokkaita on nyt myös seitsemänä tasona, yksi tiedosto kullekin (templates/candidates-*.omap). Kartta luettelee ne force-ajon jälkeen (vanha kartta säilyy tiedostona .omap.bak), tai kartoittaja lisää ne OOM:ssa (Templates > Open template…).
cli-summary-wrote-ocad = Kirjoitettu OCAD-kartta { $path } ({ $count } kohdetta)
cli-summary-kept-ocad = Säilytetty nykyinen OCAD-kartta { $path } (siinä voi olla kartoittajan työtä; force korvaa sen)
cli-summary-ocad-old-candidates = OCAD-kartta viittaa yhä tiedostoon { $path } taustakarttana; tämä ajo täytti sen kaikilla ehdokkailla. Ehdokkaita on nyt myös seitsemänä tasona, yksi tiedosto kullekin (-candidates-*.ocd). OCAD-kartta luettelee ne force-ajon jälkeen (vanha kartta säilyy tiedostona .ocd.bak), tai kartoittaja lisää ne OCADissa taustakarttoina.
cli-summary-ocad-left-out = OCAD-kartalta jätettiin pois: { $skipped } kohdetta ja { $dropped } rappeutunutta osaa
cli-summary-ocad-symbols = Valitse OCADissa Symbol > Load Colors and Symbols From… ja valitse virallinen symbolisarja{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] {" "}eli IOF:n koulukarttasarja 2019
       *[other] {" "}ISSprOM 2019
    }; se korvaa paikkamerkkisymbolit numeron mukaan.
cli-summary-ocad-failed = OCAD-vienti epäonnistui: { $error }
cli-summary-pdf-without-credits = PDF ilman lähdemainintoja: ODbL ja dl-de/by-2-0 edellyttävät, että mainitset lähteet, kun luovutat kartan eteenpäin; katso ATTRIBUTION.txt
cli-summary-wrote-pdf = Kirjoitettu kartta { $path } (ilman ehdokkaita)
cli-summary-pdf-failed = PDF-vienti epäonnistui: { $error }
cli-summary-wrote-pdf-candidates = Kirjoitettu tarkistustuloste { $path } (kartta kaikkine ehdokkaineen, paikkatarkasti)
cli-summary-pdf-candidates-failed = Tarkistustulosteen vienti epäonnistui: { $error }
cli-summary-training-failed = Harjoituskartan vienti epäonnistui: { $error }
cli-summary-kept-training = Säilytetty nykyinen harjoituskartta { $path }, jotta ajo ei korvaa tulostettua tai eteenpäin annettua karttaa; poista se tai aseta force uusiaksesi sen

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = tässä järjestelmässä ei ole kotikansiota; käytä absoluuttista polkua
mcp-error-relative-path = { $path } on suhteellinen polku; käytä absoluuttista polkua tai ~-merkillä alkavaa
mcp-error-home-folder = { $path } on itse kotikansio; käytä sen sisällä olevaa kansiota, esim. ~/maps/school
mcp-error-no-project = kansiossa { $path } ei ole freemapper-projektia; käytä ensin create_project
mcp-error-unknown-kind = tuntematon karttatyyppi { $kind }; käytä arvoa "schoolyard", "sprint" tai "forest"
mcp-error-reading = luetaan { $path }: { $error }
mcp-error-one-area-source = anna täsmälleen yksi: osm_id (find_placesta) tai geojson
mcp-error-page-open = tämän projektin alueenvalintasivu on yhä auki; pyydä käyttäjää tallentamaan alue ja painamaan sivulla Valmis, käynnistä sitten putki
mcp-error-no-job = työtä { $job } ei ole; start_pipeline ja permits_for_area palauttavat tämän istunnon työtunnukset
mcp-error-job-running = { $job } on yhä käynnissä kohteelle { $path }; kerrallaan ajetaan yksi putki, koska projektit jakavat latausvälimuistin. Käynnistä tämä, kun project_status ilmoittaa tilan finished tai failed
mcp-error-page-start = sivua ei voi käynnistää: { $error }
mcp-created-project = Projekti { $path } luotu alueelle { $area } ({ $kind }, 1:{ $scale }). Seuraavaksi: estimate_download, sitten start_pipeline.
mcp-created-project-given-area = Projekti { $path } luotu annetulle alueelle ({ $kind }, 1:{ $scale }). Seuraavaksi: estimate_download, sitten start_pipeline.
mcp-page-already-open = Tämän kansion sivu on jo auki; se säilyttää asetukset, joilla se avattiin.
mcp-page-next = Pyydä käyttäjää valitsemaan alue sivulla, painamaan Tallenna ja sitten Valmis.
mcp-preview-of = Esikatselu: { $layer }
mcp-permits-next = Lupahuomautuksia tehdään. Kysy tilaa komennolla project_status tällä job_id:llä; kun työ on valmis, permits_md sisältää tiedoston permits.md tekstin. Välitä se eteenpäin vihjeenä, ei oikeudellisena neuvontana.
mcp-tool-check-update = Onko uudempi freemapper-julkaisu olemassa: sen versio, julkaisupäivä, julkaisutiedot ja latauskoko, tai että tämä on ajan tasalla. Ei asenna mitään.
mcp-tool-create-project = Luo projektikansio kartta-alueelle. Anna osm_id (find_placesta) tai GeoJSON-tiedosto. kind on schoolyard (1:1000, oletus), sprint (1:4000) tai forest (1:15000).
mcp-param-create-project-dir = Uusi projektikansio (absoluuttinen polku tai ~-merkillä alkava).
mcp-param-create-project-geojson = osm_id:n sijaan: GeoJSON-tiedosto, jossa on kartta-alueen monikulmio (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (oletus, 1:1000), "sprint" (1:4000) tai "forest" (1:15000).
mcp-param-create-project-osm-id = Kartta-alueen OSM-kohde find_placesta, esim. "way/992423119".
mcp-param-create-project-part = osm_id:n kanssa: mitä osaa käytetään, kun alue koostuu useista erillisistä osista (1 on suurin).
mcp-param-create-project-scale = Mittakaavan nimittäjä: koulunpiha 1000 (oletus); sprintti 4000 (oletus) tai 5000; metsä 15000 (oletus), 10000 tai 7500. Muut arvot hylätään; mittakaavaa 1:3000 varten piirrä mittakaavassa 4000, freemapper-print suurentaa tulostettaessa.
mcp-param-create-project-title = Kartan otsikko reunatietoja varten (oletus: kansion nimi).
mcp-tool-estimate-download = Mitä start_pipeline lataa projektille: ruudut, kukin lataus kokoineen megatavuina, onko se välimuistissa ja mahtuuko se levylle, vielä ladattavat megatavut, sekä kohdassa disk välimuistin ja projektikansion levyjen vapaa tila ja viestit siitä, mikä ei mahdu (kerro käyttäjälle).
mcp-param-estimate-download-dir = Projektikansio (absoluuttinen polku tai ~-merkillä alkava).
mcp-tool-find-place = Hae koulua, puistoa tai muuta paikkaa nimellä (OpenStreetMap Nominatim). Palauttaa enintään 5 osumaa, alueet ensin; vain osumat, joilla on has_area, voivat olla kartta-alue. Tyhjä luettelo tarkoittaa, ettei mitään löytynyt.
mcp-param-find-place-query = Koulun tai paikan nimi, esim. "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Asenna uusin freemapper-julkaisu tämän ohjelman päälle. Kutsu vasta, kun henkilö on suostunut, arvolla confirm true. Uusi versio tulee käyttöön, kun avustaja käynnistää freemapper-palvelimen uudelleen.
mcp-param-install-update-confirm = true, kun henkilö on suostunut asentamaan uudemman version; ilman sitä mitään ei asenneta.
mcp-tool-list-layers = Projektin tasot: kartta, seitsemän ehdokastasoa ja rasteripohjat, kukin tiedostoineen, onko se jo tehty ja mikä vaihe sen tekee.
mcp-param-list-layers-dir = Projektikansio (absoluuttinen polku tai ~-merkillä alkava).
mcp-tool-open-area-selector = Avaa selaimessa sivu, jolla käyttäjä piirtää kartta-alueen (suorakulmio tai monikulmio) tai valitsee paikan, näkee ruudut ja latauksen koon ja tallentaa sen; tallennus luo projektikansion tai muuttaa sen aluetta. Palauttaa käyttäjälle annettavan URL-osoitteen. Sivu pysyy käynnissä, kunnes käyttäjä painaa Valmis; uusi kutsu samalle kansiolle palauttaa saman URL-osoitteen.
mcp-param-open-area-selector-dir = Projektikansio (absoluuttinen polku tai ~-merkillä alkava); luodaan, kun käyttäjä tallentaa.
mcp-param-open-area-selector-kind = Uuden projektin karttatyyppi: "schoolyard" (oletus, 1:1000), "sprint" (1:4000) tai "forest" (1:15000).
mcp-param-open-area-selector-ocad = Ajopaneeli käynnistyy valinnalla OCAD-aloitusprojekti. Oletus false.
mcp-param-open-area-selector-offline = Ajopaneeli käynnistyy valinnalla "Vain välimuisti, ei verkkoa". Oletus false.
mcp-param-open-area-selector-open-browser = Avaa sivu myös tämän tietokoneen oletusselaimessa. Oletus true.
mcp-param-open-area-selector-pdf = Ajopaneeli käynnistyy valinnalla tulostettava PDF. Oletus false.
mcp-param-open-area-selector-pdf-candidates = Ajopaneeli käynnistyy valinnalla tarkistustuloste (kaikki ehdokkaat). Oletus false.
mcp-param-open-area-selector-pdf-scale-bar = Ajopaneeli käynnistyy valinnalla mittakaavajana PDF-tiedostoihin. Oletus false.
mcp-param-open-area-selector-pdf-sources = Lähderivit PDF-tiedostoissa, joilla paneeli käynnistyy: "auto" (oletus), "full" tai "credit-only", kuten start_pipelinessa.
mcp-param-open-area-selector-permits = Ajopaneeli käynnistyy valinnalla lupahuomautukset. Oletus false.
mcp-param-open-area-selector-scale = Uuden projektin mittakaavan nimittäjä: koulunpiha 1000 (oletus); sprintti 4000 (oletus) tai 5000; metsä 15000 (oletus), 10000 tai 7500. Muut arvot hylätään.
mcp-param-open-area-selector-title = Uuden projektin kartan otsikko.
mcp-param-open-area-selector-training = Ajopaneeli käynnistyy valinnalla harjoituskartta. Oletus false.
mcp-tool-preview-layer = Tason PNG-esikatselu (enintään 800 px): map (viimeisimmän ajon kirjoittama kuva karttakohteista, ilman ehdokkaita) tai rasteripohja: orthophoto, hillshade, slope, relief, ndsm, tai k5 Berliinissä.
mcp-param-preview-layer-dir = Projektikansio (absoluuttinen polku tai ~-merkillä alkava).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" tai "k5" (vain Berliini).
mcp-tool-project-status = Työn tila (start_pipeline tai permits_for_area): running, finished tai failed, viimeiset edistymisrivit, varoitukset, kirjoitetut tiedostot sekä tulos tai virhe. Valmis permits_for_area -työ sisältää myös tiedoston permits.md tekstin kentässä permits_md.
mcp-param-project-status-job-id = Työtunnus komennosta start_pipeline tai permits_for_area, esim. "job-1".
mcp-tool-permits-for-area = Tee projektin lupahuomautukset: kartta-alueen hallinnolliset yksiköt, siinä olevat suojelualueet ja kuka maata hallinnoi, sekä viranomaiset, joilta kysyä; vihje, ei oikeudellista neuvontaa. Kirjoittaa tiedostot permits.md ja permits.json kansioon. Toimii taustalla ja palauttaa job_id:n; kysy tilaa komennolla project_status, jonka vastaus sisältää työn valmistuttua tiedoston permits.md tekstin kentässä permits_md. Kerrallaan ajetaan yksi työ.
mcp-param-permits-for-area-dir = Projektikansio (absoluuttinen polku tai ~-merkillä alkava).
mcp-param-permits-for-area-offline = Lue vain aiemman ajon projektiin tallentamat tiedot ilman verkkoyhteyttä. Oletus false.
mcp-tool-start-pipeline = Lataa tiedot, laske pohjat ja kirjoita aloitusprojekti (fetch, derive, export). Toimii taustalla ja palauttaa job_id:n; kysy tilaa komennolla project_status. Kerrallaan ajetaan yksi putki. Se säilyttää olemassa olevan kartan, ellei force ole true.
mcp-param-start-pipeline-dir = Projektikansio (absoluuttinen polku tai ~-merkillä alkava).
mcp-param-start-pipeline-force = Korvaa olemassa oleva karttatiedosto ja säilytä vanha tiedostona .omap.bak. Oletus false.
mcp-param-start-pipeline-ocad = Kirjoita myös OCAD-aloituspiste kansioon ocad/ (OCAD 12 -muoto, OCADissa muokkaaville käyttäjille). Oletus false.
mcp-param-start-pipeline-offline = Käytä vain latausvälimuistia ilman verkkoyhteyttä. Oletus false.
mcp-param-start-pipeline-pdf = Kirjoita myös <name>.pdf, tulostusvalmis kartta (vain karttakohteet, ei ehdokkaita). Oletus false.
mcp-param-start-pipeline-pdf-candidates = Kirjoita myös <name>-candidates.pdf, tarkistustuloste: kartta kaikkine ehdokkaineen, paikkatarkasti (tarkista ne maastossa). Oletus false.
mcp-param-start-pipeline-pdf-scale-bar = Piirrä mittakaavajana PDF-tiedostoihin. Oletus false (saksalaisissa kartoissa sitä ei yleensä ole; mittakaavaluku jää otsikkolohkoon).
mcp-param-start-pipeline-pdf-sources = Lähderivit PDF-tiedostoissa: "auto" (oletus; koko luettelo, lyhennetään vain jos se veisi yli 5 riviä), "full" (ei lyhennetä koskaan) tai "credit-only" (vain freemapperin lähderivi; ODbL ja dl-de/by-2-0 edellyttävät silloin lähteiden mainitsemista, kun kartta luovutetaan eteenpäin). ATTRIBUTION.txt luettelee aina kaikki lähteet.
mcp-param-start-pipeline-permits = Kirjoita myös lupahuomautukset (permits.md ja permits.json) kartan jälkeen: hallinnolliset yksiköt, suojelualueet ja kuka maata hallinnoi. Epäonnistuminen siinä on varoitus, ei epäonnistunut työ. Oletus false.
mcp-param-start-pipeline-training = Kirjoita myös harjoituskartta <name>-training.omap ja sen PDF <name>-training.pdf: kartta ja kaikki ehdokkaat siivottuna tulostettavaksi sellaisenaan; arvolla ocad myös ocad/<name>-training.ocd. Olemassa oleva harjoituskartta säilyy, ellei force ole true. Oletus false.
mcp-param-start-pipeline-training-views = Arvon training kanssa: kirjoita myös harjoituskartta ilman sitä, minkä näkymä jättää pois, yksi tiedostojoukko kutakin merkintää kohden: "no-paths", "brown-only", "no-north-lines", tai luettelo, kuten "no:505,506" tai "only:101,102", yhdistettynä merkillä +. Tiedostot ovat <name>-training-<view>.omap ja sen PDF.
mcp-instructions = freemapper tekee OpenOrienteering Mapper -aloitusprojekteja suunnistuskartoille (koulunpiha 1:1000, sprintti 1:4000, metsä 1:15000) alueen { $states } virallisista paikkatiedoista ja OSM:stä. Työnkulku: find_place (hae koulu tai alue nimellä; valitse osuma, jolla on has_area) → create_project (uusi kansio, absoluuttinen polku, osm_id:llä) → estimate_download (kerro käyttäjälle, kuinka paljon ladataan) → start_pipeline (palauttaa työtunnuksen; lataus voi kestää minuutteja) → project_status (kysy tilaa muutaman sekunnin välein, kunnes tila on finished tai failed) → list_layers ja preview_layer (näytä käyttäjälle pohjat). Käyttäjä avaa sitten karttatiedoston OpenOrienteering Mapperissa. Kun mikään paikka ei sovi tai käyttäjä haluaa piirtää alueen, open_area_selector avaa selaimessa sivun, jolla käyttäjä piirtää tai valitsee alueen ja tallentaa sen (tämä luo projektin); jatka komennolla estimate_download, kun käyttäjä kertoo sen olevan tallennettu. Polkujen on oltava absoluuttisia tai alettava merkillä ~. Kerrallaan ajetaan yksi putki. permits_for_area tekee projektin lupahuomautukset (hallinnolliset yksiköt, suojelualueet, kuka maata hallinnoi; vihje, ei oikeudellista neuvontaa) työnä kuten start_pipeline. check_update kertoo, onko uudempi freemapper olemassa; install_update asentaa sen, vain arvolla confirm: true henkilön suostuttua, ja avustaja käynnistää sitten freemapper-palvelimen uudelleen.

cli-contour-interval-saved = Korkeuskäyrien väli tallennettu tiedostoon freemapper.toml: { $interval } m; derive jäljittää korkeuskäyrät sillä
cli-contour-interval-auto = Korkeuskäyrien väli tiedostossa freemapper.toml: automaattinen (karttatyypin oma; metsäkartalla 2,5 m tasaisessa maastossa ja 10 m jyrkässä)
cli-steep-slope-saved = Jyrkkyysraja tallennettu tiedostoon freemapper.toml: { $slope }°; derive etsii sillä penkereitä ja kallioita
cli-steep-slope-auto = Jyrkkyysraja tiedostossa freemapper.toml: oletus (noin 27°)
help-arg-contour-interval = Korkeuskäyrien väli metreinä, esim. `2.5`, tai `auto` karttatyypin omalle (metsäkartta: 5 m, 2,5 m tasaisessa, 10 m jyrkässä). Tallennetaan tiedostoon freemapper.toml; derive jäljittää korkeuskäyrät sillä
help-arg-steep-slope = Rinteen kaltevuuskulma asteina, josta alkaen maasto katsotaan jyrkäksi penkereitä ja kallioita etsittäessä, esim. `55`, tai `auto` oletukselle 27°. Nosta sitä (55 - 65) kalliolabyrinteissä ja rotkoissa, joissa lähes jokainen rinne on jyrkkä ja penkereet ja kalliot sulautuvat yhdeksi laikuksi. Tallennetaan tiedostoon freemapper.toml; derive etsii sillä penkereitä ja kallioita
cli-content-saved = Sisältöasetus tallennettu tiedostoon freemapper.toml: { $summary }
cli-content-invalid = Sisältöasetusta ei voi lukea: { $detail }
help-arg-content = Mikä tulee suoraan kartalle: `safe` jättää jokaisen kohteen sinne, minne ajo sen sijoittaa, `balanced` (oletus) tuo kiinteistörekisterin ja rekisterien kohteet kartalle, `full` tuo kaiken kartalle. Valinta tallennetaan tiedostoon freemapper.toml
help-arg-content-set = Lähetä kohderyhmä paikkaan `map`, `candidates` tai `off`, muodossa `GROUP=PLACE`, esim. `walls=map`; voi toistaa. Ryhmät: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Tallennetaan tiedostoon freemapper.toml
mcp-param-create-project-content = Mikä tulee suoraan kartalle: "safe", "balanced" (oletus) tai "full". Tallennetaan projektiin. balanced tuo kiinteistörekisterin ja rekisterien kohteet kartalle, full tuo kaiken.
mcp-param-create-project-content-set = Ryhmät, jotka lähetetään muualle kuin profiili sanoo, muodossa "group=place", jossa place on map, candidates tai off, esim. "walls=map". Ryhmät: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Korkeuskäyrien väli metreinä, esim. 2.5; 0 palaa automaattiseen valintaan (karttatyypin oma; metsäkartalla 2,5 m tasaisessa maastossa ja 10 m jyrkässä). Tallennetaan projektiin; seuraava derive jäljittää korkeuskäyrät sillä.
mcp-param-create-project-steep-slope = Rinteen kaltevuuskulma asteina, josta alkaen maasto katsotaan jyrkäksi penkereitä ja kallioita etsittäessä, esim. 55; 0 palauttaa oletuksen 27°. Nosta sitä (55 - 65) kalliolabyrinteissä ja rotkoissa, joissa lähes jokainen rinne on jyrkkä ja penkereet ja kalliot sulautuvat yhdeksi laikuksi. Tallennetaan projektiin; seuraava derive etsii sillä penkereitä ja kallioita.
mcp-param-start-pipeline-content = Muuta ennen ajoa, mikä tulee suoraan kartalle: "safe", "balanced" tai "full". Tallennetaan projektiin. Oletus: projektin asetus, muuten balanced.
mcp-param-start-pipeline-content-set = Ryhmät, jotka lähetetään muualle kuin profiili sanoo, muodossa "group=place", jossa place on map, candidates tai off, esim. "fences=candidates". Tallennetaan projektiin.
mcp-param-start-pipeline-contour-interval = Korkeuskäyrien väli metreinä, esim. 2.5; 0 palaa automaattiseen valintaan (karttatyypin oma; metsäkartalla 2,5 m tasaisessa maastossa ja 10 m jyrkässä). Tallennetaan projektiin; seuraava derive jäljittää korkeuskäyrät sillä.
mcp-param-start-pipeline-steep-slope = Rinteen kaltevuuskulma asteina, josta alkaen maasto katsotaan jyrkäksi penkereitä ja kallioita etsittäessä, esim. 55; 0 palauttaa oletuksen 27°. Nosta sitä (55 - 65) kalliolabyrinteissä ja rotkoissa, joissa lähes jokainen rinne on jyrkkä ja penkereet ja kalliot sulautuvat yhdeksi laikuksi. Tallennetaan projektiin; seuraava derive etsii sillä penkereitä ja kallioita.
mcp-error-content = Sisältöasetusta ei voi lukea: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Kirjoita projektin lupahuomautukset (permits.md ja permits.json): hallinnolliset yksiköt, suojelualueet ja hallinnoijat, joita kartta-alue koskettaa, sekä se, keneltä lupaa yleensä pitää kysyä. Se kysyy BfN:ltä ja BKG:ltä ja, kun kartta-alueen osavaltiolla on omia avoimia palveluja, niiltä sovittimensa palvelinten kautta (luettelo oppaan luvussa 12); Nordrhein-Westfalenissa se lataa tätä varten ensimmäisellä kerralla noin 32 MB, Brandenburgissa noin 4 MB, myöhemmin vain uudemmat tiedostot. Se lukee jo välimuistissa olevan OSM-otteen; --offline käyttää aiemmin haettua
help-permits-dir = Projektikansio
help-permits-offline = Käytä vain aiemmin haettuja tietoja, ei verkkoyhteyttä
help-run-permits = Kirjoita myös lupahuomautukset (permits.md ja permits.json) kartan jälkeen; kysyy BfN:ltä, BKG:ltä ja osavaltion avoimilta palveluilta (opas, luku 12). Epäonnistuminen siinä on varoitus, ja kartta säilyy
cli-wrote-permits = Kirjoitettu lupahuomautukset { $path }
cli-summary-wrote-permits = Kirjoitettu lupahuomautukset { $path }
cli-permits-failed = lupahuomautuksia ei voitu kirjoittaa: { $error }
cli-permits-source-left-out = lupahuomautukset jättävät pois lähteen { $source }: { $reason }
cli-permits-not-read = sitä ei luettu
error-permits-stored = { $path }: { $message }; hae lupahuomautukset uudelleen verkossa
error-permits-nothing-read = yhtään lupahuomautusten lähdettä ei voitu lukea ({ $missing }); hae ne ensin verkossa
error-permits-missing-not-read = { $source }: ei luettu

permits-title = Lupahuomautukset
permits-disclaimer = Vihje, ei oikeudellista neuvontaa. Toimivaltaiset viranomaiset päättävät. Tietojen päivämäärän ja tarkkuuden löydät taulukosta kohdassa "Lähteet ja tietojen päivämäärä"; alueita voi puuttua tai ne voivat olla vanhentuneita (BfN: yhdistetään kerran vuodessa, "Nicht für Planungszwecke geeignet.", eli ei sovellu suunnitteluun).
permits-disclaimer-regional = Vihje, ei oikeudellista neuvontaa. Toimivaltaiset viranomaiset päättävät. Tietojen päivämäärän ja tarkkuuden löydät taulukosta kohdassa "Lähteet ja tietojen päivämäärä"; alueita voi puuttua tai ne voivat olla vanhentuneita.
permits-bfn-merge = BfN yhdistää osavaltioiden tiedot kerran vuodessa ja merkitsee ne soveltumattomiksi suunnitteluun. Osavaltion viranomaiset päättävät.
permits-created = Laadittu { $date } { $kind ->
        [schoolyard] koulunpihakartan
        [forest] metsäkartan
       *[other] sprinttikartan
    } kartta-alueelle ({ $area } ha).
permits-section-admin = Hallinnolliset yksiköt
permits-section-protected = Suojelualueet (BfN)
permits-section-osm-protected = Lisää suojelualueita OpenStreetMapista
permits-section-operators = Hallinnoijat ja omistajat OpenStreetMapin mukaan
permits-section-hints = Vihjeet
permits-section-sources = Lähteet ja tietojen päivämäärä
permits-none-admin = VG250 ei nimeä kartta-alueelle yhtään hallinnollista yksikköä.
permits-none-protected = BfN:n mukaan kartta-alueella ei ole suojelualuetta.
permits-none-osm-protected = OpenStreetMap ei nimeä kartta-alueelta suojelualuetta, jota BfN ei jo luettelisi.
permits-none-operators = OpenStreetMap ei nimeä kartta-alueelle hallinnoijaa eikä omistajaa.
permits-none-hints = Ei vihjeitä.
permits-section-not-read = Lähteen { $source } tietoja ei luettu, joten tämä osio on tyhjä, vaikka merkintöjä voisi olla.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] BKG:n hallinnolliset alueet 1:250 000 (VG250)
        [bfn] BfN:n Saksan suojelualueet
       *[osm] OpenStreetMap, otteesta, jota freemapper säilyttää välimuistissaan
    }
permits-level = { $level ->
        [land] Osavaltio (Land)
        [kreis] Piiri (Kreis)
        [verwaltungsgemeinschaft] Amt tai Verwaltungsgemeinschaft
       *[gemeinde] Kunta (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent } %
permits-share-below-one = < 1 %
permits-at-edge = Koskettaa reunaa (alle 0,5 % kartta-alueesta):
permits-at-edge-generalised = VG250 on yleistetty mittakaavaan 1:250 000. Yksikkö, joka vain koskettaa reunaa, voi johtua siitä.
permits-kind = { $kind ->
        [nationalpark] Kansallispuisto
        [nationales-naturmonument] Kansallinen luonnonmuistomerkki
        [naturschutzgebiet] Luonnonsuojelualue (Naturschutzgebiet)
        [ffh] FFH-alue (Natura 2000)
        [vogelschutzgebiet] Lintujensuojelualue (Natura 2000)
        [biosphaerenreservat] Biosfäärialue
        [biosphaerenreservat-zone] Biosfäärialueen vyöhyke
        [landschaftsschutzgebiet] Maisemansuojelualue (Landschaftsschutzgebiet)
       *[naturpark] Luonnonpuisto
    }
permits-protected-line = { $kind } "{ $name }", { $share }
permits-protected-date = Perustettu { $date }
permits-protected-title = Oikeudellinen peruste: { $title }
permits-protected-link = Lakiteksti: { $url }
permits-protected-site-link = Kohteen sivu (Natura 2000): { $url }
permits-protected-year = Tietojen vuosi BfN:llä: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = suojeluluokka { $class }
permits-quoted = "{ $text }"
permits-osm-unnamed = ilman nimeä
permits-osm-operator = Hallinnoija OSM:n mukaan: { $operator }
permits-osm-enclosing-gap = OpenStreetMapin alueet, jotka ympäröivät koko kartta-alueen ilman että yksikään kulma on sen sisällä, voivat puuttua tästä, samoin suojelualueet, jotka on kartoitettu OSM:ään relaationa `type=boundary`. BfN:n ja VG250:n osat eivät ole tästä kärsineet.
permits-osm-enclosing-gap-short = OpenStreetMapin alueet, jotka ympäröivät koko kartta-alueen ilman yhtään kulmaa sen sisällä, voivat puuttua tästä.
permits-operator-line = { $name } ({ $roles ->
        [operator] hallinnoija
        [owner] omistaja
       *[both] hallinnoija ja omistaja
    }): { $objects }
permits-column-source = Lähde
permits-column-state = Tila
permits-column-credit = Lähdemaininta ja lisenssi
permits-state-read = luettu, haettu { $date }
permits-state-read-osm = luettu, OSM-ote päivältä { $date }
permits-state-unreachable = ei tavoitettavissa ({ $reason })
permits-state-not-read = ei luettu
permits-no-credit = ei käytetty
permits-hint-city-state = Kartta-alue on Berliinissä tai Hampurissa. Siellä vastuussa on yleensä Bezirksamt (piirihallinto) tai osavaltion viranomainen. Nämä huomautukset eivät vielä määritä Bezirkiä.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Suojelualueet koskevat alempaa luonnonsuojeluviranomaista (untere Naturschutzbehörde). Tässä se on kaupunki { $kreis } itse.
        [no] Suojelualueet koskevat alempaa luonnonsuojeluviranomaista (untere Naturschutzbehörde), yleensä { $kreis }.
       *[unknown] Suojelualueet koskevat toimivaltaista alempaa luonnonsuojeluviranomaista (untere Naturschutzbehörde), yleensä Landkreisia tai kreisfreie Stadtia.
    }
permits-hint-national-park = Kansallispuistolla on omat sääntönsä. Kysy kansallispuiston hallinnolta ajoissa.
permits-hint-nsg-paths = Luonnonsuojelualueella on yleensä pysyttävä poluilla. Jos haluat juosta polkujen ulkopuolella, kysy luonnonsuojeluviranomaiselta.
permits-hint-natura2000 = FFH-alue tai lintujensuojelualue (Natura 2000) koskettaa kartta-aluetta. Tapahtumaa varten luonnonsuojeluviranomainen voi pyytää vaikutusten arviointia.
permits-hint-protected-areas-pl = Kartta-alueella on luonnonsuojelun suojelualueita tai suojelukohteita. Kansallispuiston, maisemapuiston tai luonnonsuojelualueen osalta yleensä puiston tai suojelualueen hallinto on oikea yhteyspaikka; muiden alueiden ja kohteiden osalta voivodikunnan alueellinen ympäristönsuojelun johtokunta (RDOŚ) osaa yleensä kertoa, kuka vastaa. Kysy ajoissa, kun suunnittelet tapahtumaa sinne.
permits-hint-order-office = { $amt ->
        [none] Katujen, aukioiden ja muun yleisen maan osalta vastuussa on yleensä kunnan { $gemeinde } Ordnungsamt (järjestysvirasto).
       *[other] Katujen, aukioiden ja muun yleisen maan osalta vastuussa on yleensä Ordnungsamt (järjestysvirasto): { $amt }, kunnalle { $gemeinde }.
    }
permits-hint-park-operator = Kartta-alueen puiston tai puutarhan hallinnoija OpenStreetMapin mukaan: { $operator }. Kysy heiltä ennen käyttöä.
permits-hint-school-operator = Koulun tai laitoksen hallinnoija OpenStreetMapin mukaan: { $operator }. Sen alueen osalta kysy koulun rehtorilta tai tältä taholta.
permits-section-districts = Bezirke (piirit, { $source })
permits-section-green-spaces = Viheralueet ({ $source })
permits-section-owners = Osavaltion ja liittovaltion (Bund) maa ({ $source })
permits-section-monuments = Muistomerkit ({ $source })
permits-section-protected-objects = Luonnonmuistomerkit ja muut suojellut kohteet ({ $source })
permits-none-districts = Lähteen { $source } mukaan kartta-alue ei ole missään Bezirkissä.
permits-none-green-spaces = Lähteen { $source } mukaan kartta-alueella ei ole yleistä viheraluetta.
permits-none-owners = Lähteen { $source } mukaan kartta-alueella ei ole osavaltion eikä liittovaltion (Bund) tilaa.
permits-none-monuments = Lähteen { $source } mukaan kartta-alueella ei ole suojeltua muistomerkkiä.
permits-none-protected-objects = Lähteen { $source } mukaan mikään sen sisältämistä suojelluista kohteista ei ole kartta-alueella.
permits-none-sampled = Kysytyissä pisteissä { $source } ei nimeä täällä mitään.
permits-gaps = Lähteen { $source } tiedot eivät ole täydelliset: { $gaps }.
permits-sampled = { $source } vastaa vain yksittäisille pisteille. freemapper kysyi pisteruudukosta kartan yli: osuudet ovat arvioita, ja pisteiden väliin jäävä voi puuttua.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = käyttötarkoitus: { $dedication }
permits-playgrounds-none = Kartta-alueella ei ole yleistä leikkipuistoa.
permits-playgrounds-one = Kartta-alueella on yksi yleinen leikkipuisto. Sitä ei luetella erikseen.
permits-playgrounds-many = Kartta-alueella on { $count } yleistä leikkipuistoa. Niitä ei luetella yksitellen.
permits-owner-other = Muut julkiset omistajat
permits-owners-rest = Muun alueen osalta tiedot eivät nimeä omistajaa.
permits-monuments-total = Muistomerkit kokonaan tai osittain kartta-alueella: { $count }.{ $links ->
        [yes] {" "}Linkit johtavat niiden merkintöihin muistomerkkiluettelossa.
       *[no] {""}
    }
permits-monuments-links-left = { $count } merkintää lisää on vain laskettu tähän, ilman linkkiä.
permits-object-kind = { $kind ->
        [natural-monument] Luonnonmuistomerkki (Naturdenkmal)
        [protected-landscape-part] Suojeltu maisemaelementti (geschützter Landschaftsbestandteil)
        [biotope] Lailla suojeltu biotooppi (geschütztes Biotop)
        [national-park] Kansallispuisto (park narodowy)
        [park-buffer-zone] Puiston suojavyöhyke (otulina)
        [landscape-park] Maisemapuisto (park krajobrazowy)
        [nature-reserve] Luonnonsuojelualue (rezerwat przyrody)
        [protected-landscape-area] Suojeltu maisema-alue (obszar chronionego krajobrazu)
        [natura2000-habitats] Natura 2000 -alue elinympäristöille (SOO)
        [natura2000-birds] Natura 2000 -alue linnuille (OSO)
        [ecological-site] Ekologinen alue (użytek ekologiczny)
        [landscape-complex] Luonto- ja maisemakokonaisuus (zespół przyrodniczo-krajobrazowy)
        [documentation-site] Dokumentointikohde (stanowisko dokumentacyjne)
        [natural-monument-pl] Luonnonmuistomerkki (pomnik przyrody)
       *[special-rules] Alue erityissäännöin
    }
permits-object-count = { $count } kohdetta
permits-object-citations = Oikeudelliset perusteet:
permits-source-note = { $source }: "{ $note }"
permits-hint-district-office = { $offices ->
        [hamburg] Katujen, aukioiden ja viheralueiden osalta Bezirkissä { $district } vastuussa on yleensä Bezirksamt { $district }, siellä Fachamt Management des öffentlichen Raumes (yleisen tilan hallinnon yksikkö).
       *[berlin] Katujen, aukioiden ja viheralueiden osalta Bezirkissä { $district } vastuussa ovat yleensä Bezirksamt { $district } -viraston Ordnungsamt (järjestysvirasto) ja Straßen- und Grünflächenamt (katu- ja viheralueyksikkö).
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] Kartta-alueen muistomerkkien osalta vastuussa on Denkmalschutzamt Hamburg (muistomerkkien suojeluvirasto).
       *[berlin] Bezirkin { $district } muistomerkkien osalta vastuussa on Bezirksamt { $district } -viraston alempi muistomerkkiviranomainen (untere Denkmalschutzbehörde).
    }
permits-hint-monument-lower-authority = { $known ->
        [no] Tässä käytetyt tiedot eivät kerro varmasti, onko kartta-alueella suojeltuja muistomerkkejä.
       *[yes] Kartta-alueella on suojeltuja muistomerkkejä.
    } Koskettaako juoksu niitä, esimerkiksi rastilla muistomerkillä tai välillä suojellun kohteen läpi, ratkaisee alempi muistomerkkiviranomainen (untere Denkmalschutzbehörde){ $kreisfrei ->
        [yes] . Tässä se on kaupunki { $kreis } itse.
        [no] , yleensä { $kreis }{ $gemeinde ->
            [none] .
           *[other] ; joissakin osavaltioissa se on kunta { $gemeinde }.
        }
       *[unknown] {" "}(yleensä Landkreis tai kreisfreie Stadt, joissakin osavaltioissa kunta).
    }
permits-hint-nature-object-lower-authority = { $kreisfrei ->
        [yes] Kartalla olevat luonnonmuistomerkit, suojellut maisemaelementit ja suojellut biotoopit kuuluvat alemmalle luonnonsuojeluviranomaiselle (untere Naturschutzbehörde). Tässä se on kaupunki { $kreis } itse.
        [no] Kartalla olevat luonnonmuistomerkit, suojellut maisemaelementit ja suojellut biotoopit kuuluvat alemmalle luonnonsuojeluviranomaiselle (untere Naturschutzbehörde), yleensä { $kreis }.
       *[unknown] Kartalla olevat luonnonmuistomerkit, suojellut maisemaelementit ja suojellut biotoopit kuuluvat alemmalle luonnonsuojeluviranomaiselle (untere Naturschutzbehörde), yleensä Landkreisille tai kreisfreie Stadtille.
    }
permits-hint-biotope = Lailla suojeltuja biotooppeja on kartta-alueella{ $names ->
        [none] {""}
       *[other] {" "}({ $names })
    }. Kaikki, mikä voi tuhota ne tai vahingoittaa niitä merkittävästi, on kielletty (§ 30 BNatSchG). Sijoita rastit ja rastivälit niiden ulkopuolelle tai kysy alemmalta luonnonsuojeluviranomaiselta (untere Naturschutzbehörde).
permits-hint-nature-object-authority = { $offices ->
        [hamburg] Luonnonmuistomerkkien ja suojeltujen maisemaelementtien osalta Bezirkissä { $district } vastuussa on yleensä Bezirksamt { $district }.
       *[berlin] Luonnonmuistomerkkien ja suojeltujen maisemaelementtien osalta Bezirkissä { $district } vastuussa on Bezirksamt { $district } -viraston alempi luonnonsuojeluviranomainen (untere Naturschutzbehörde).
    }
permits-hint-owner-share = { $unnamed ->
        [yes] Osavaltion avoimen datan mukaan { $share } kartta-alueesta kuuluu muille julkisille omistajille.
       *[no] Osavaltion avoimen datan mukaan { $share } kartta-alueesta kuuluu omistajalle "{ $owner }".
    }
permits-hint-owners-unknown = { $rest ->
        [yes] Muun alueen omistajia ei voi selvittää avoimesta datasta. Oikeutetulla edulla (esimerkiksi järjestäjänä) voit pyytää otteen kiinteistörekisteristä (Grundbuchamt); julkisen maan osalta toimivaltainen Ordnungsamt tai puistoyksikkö auttaa.
       *[no] Näiden alueiden omistajia ei voi selvittää avoimesta datasta. Oikeutetulla edulla (esimerkiksi järjestäjänä) voit pyytää otteen kiinteistörekisteristä (Grundbuchamt); julkisen maan osalta toimivaltainen Ordnungsamt tai puistoyksikkö auttaa.
    }
permits-section-forest-units = Metsähallinto ({ $source })
permits-section-state-forest = Valtion metsä ({ $source })
permits-none-forest-units = Lähteen { $source } mukaan kartta-alue ei ole minkään metsätoimiston alueella.
permits-section-admin-units = Hallintoyksiköt ({ $source })
permits-none-admin-units = Lähteen { $source } mukaan karttaalue ei ole minkään rekisterin hallintoyksikön alueella.
permits-section-register-monuments = Kansallisen rekisterin muistomerkit ({ $source })
permits-none-register-monuments = Lähteen { $source } mukaan karttaalueen ympärillä olevassa suorakulmiossa ei ole rekisteröityä muistomerkkiä.
permits-register-monuments-total = Rekisteröidyt kohteet karttaalueen ympärillä olevassa suorakulmiossa: { $count }. Rekisteri ei anna sijaintia, joten nämä huomautukset eivät voi kertoa, missä ne ovat; se on tarkistettava kartalta.
permits-register-monuments-left = { $count } muuta on tässä vain laskettu, ilman nimiä.
permits-section-register-archaeology = Kansallisen rekisterin arkeologiset kohteet ({ $source })
permits-none-register-archaeology = Lähteen { $source } mukaan karttaalueen ympärillä olevassa suorakulmiossa ei ole rekisteröityä arkeologista kohdetta.
permits-register-archaeology-total = Rekisteröidyt arkeologiset kohteet karttaalueen ympärillä olevassa suorakulmiossa: { $count }. Rekisteri ei anna sijaintia, joten nämä huomautukset eivät voi kertoa, missä ne ovat; se on tarkistettava kartalta.
permits-register-archaeology-left = { $count } muuta on tässä vain laskettu, ilman nimiä.
permits-section-register-history = Historian muistomerkit ({ $source })
permits-none-register-history = Lähteen { $source } mukaan karttaalueen ympärillä olevassa suorakulmiossa ei ole yhtään historian muistomerkkiä.
permits-register-history-in-area = Historian muistomerkit karttaalueella: { $count }.
permits-register-history-nearby = Lisää historian muistomerkkejä karttaalueen ympärillä olevassa suorakulmiossa, mutta ei karttaalueella: { $count }.
permits-register-history-left = { $count } muuta on tässä vain laskettu, ilman nimiä.
permits-admin-unit-code = { $name } (TERYT { $code })
permits-none-state-forest = Lähteen { $source } mukaan kartta-alueella ei ole valtion metsää.
permits-forest-units-reach = Metsätoimisto vastaa alueensa metsästä. Sen alue voi kattaa myös kaupunkeja ja peltoja.
permits-state-forest-office = vastuussa: { $office }
permits-hint-forest-office = { $units ->
        [none] Kartta-alueen metsästä vastaava metsätoimisto: { $office } ({ $administration }). Kysy sieltä ajoissa, kun suunnittelet tapahtumaa metsässä.
       *[other] Kartta-alueen metsästä vastaava metsätoimisto: { $office } ({ $administration }), { $units }. Kysy sieltä ajoissa, kun suunnittelet tapahtumaa metsässä.
    }
permits-hint-forest-owner = { $office ->
        [known] Tapahtuma metsässä vaatii metsänomistajien suostumuksen. Metsätoimisto osaa yleensä kertoa, kuka metsän omistaa. Useissa osavaltioissa järjestetystä tapahtumasta metsässä on myös ilmoitettava metsäviranomaiselle tai saatava sen lupa.
       *[unknown] Tapahtuma metsässä vaatii metsänomistajien suostumuksen. Useissa osavaltioissa järjestetystä tapahtumasta metsässä on myös ilmoitettava metsäviranomaiselle tai saatava sen lupa. Nämä huomautukset eivät nimeä tässä vastuussa olevaa metsätoimistoa; osavaltion metsähallinto tai alempi metsäviranomainen osaa kertoa.
    }
permits-hint-forest-owner-pl = { $office ->
        [known] Metsässä järjestettävä tapahtuma tarvitsee yleensä metsänomistajan suostumuksen. Edellä mainittu metsäpiiri (Nadleśnictwo) osaa yleensä kertoa, kuka metsästä huolehtii. Metsä, joka ei kuulu valtion metsille, esimerkiksi yksityismetsä, ei ole näiden huomautusten tiedoissa. Metsää, joka ei kuulu valtiolle, esimerkiksi yksityis- tai kunnan metsää, valvoo metsätaloudessa starosta (starosta) (metsälaki, art. 5 ust. 1 pkt 2); kysy siellä toimistosta (starostwo powiatowe).
       *[unknown] Metsässä järjestettävä tapahtuma tarvitsee yleensä metsänomistajan suostumuksen. Nämä huomautukset eivät nimeä kartta-alueelle yhtään valtion metsien metsäpiiriä; tiedot kattavat vain valtion metsät, joten metsä voi kuulua jollekin muulle, esimerkiksi yksityiselle omistajalle. Omistaja on selvitettävä erikseen. Metsää, joka ei kuulu valtiolle, esimerkiksi yksityis- tai kunnan metsää, valvoo metsätaloudessa starosta (starosta) (metsälaki, art. 5 ust. 1 pkt 2); kysy siellä toimistosta (starostwo powiatowe).
    }
permits-hint-monument-register-pl = Kansallisen rekisterin kohteita on karttaalueen ympärillä olevassa suorakulmiossa. Niiden sijainteja ei ole tiedoissa, joten tarkista kartalta, missä ne ovat. Muistomerkkien suojelulain (ustawa o ochronie zabytków) 36 artiklan 1 momentin mukaan rekisteröidyn kohteen konservointi-, entisöinti- ja rakennustöihin tarvitaan voivodikunnan muinaismuistovirkailijan (Wojewódzki Konserwator Zabytków) lupa. 3.6.2026 alkaen kohteen ympäristön rakennustöihin, käyttötarkoituksen muutokseen, siihen kiinnitettäviin laitteisiin tai kylttiin ja muihin toimiin, jotka voisivat muuttaa sen ainesta tai ulkonäköä, tarvitaan vain ilmoitus virkailijalle (36aa artikla); virkailija voi 60 päivän kuluessa vastustaa ja vaatia silloin luvan. Laki ei mainitse juoksua tai rastia sellaisenaan; kysy virkailijan toimistosta, jos tapahtuma koskee kohdetta tai sen ympäristöä.
permits-hint-archaeology-register-pl = Kansallisen rekisterin arkeologisia kohteita on karttaalueen ympärillä olevassa suorakulmiossa. Niiden sijainteja ei ole datassa, joten tarkista kartalta, missä ne ovat. Muistomerkkien suojelulain (ustawa o ochronie zabytków) 36 artiklan 1 momentin mukaan arkeologisiin tutkimuksiin (pkt 5), kätkettyjen tai hylättyjen irtainten muinaisesineiden etsimiseen elektronisilla tai teknisillä laitteilla, kuten metallinilmaisimilla (pkt 12), sekä rekisteröidyn kohteen konservointi-, entisöinti- tai rakennustöihin (pkt 1) tarvitaan voivodikunnan muistomerkkien suojelijan (Wojewódzki Konserwator Zabytków) lupa. Etsintälupa poistuu 1.1.2027: 36b artikla sallii silloin täysi-ikäisen etsiä kiinteistön omistajan ja haltijan suostumuksella, kun etsintä on ilmoitettu etsintärekisteriin kulttuuriministeriön sovelluksella, ja 36c artikla kieltää tällaisen etsinnän muun muassa rekisteröidyillä kohteilla ja alle 10 m:n etäisyydellä niistä. Laki ei mainitse kävelyä, juoksua tai rastia sellaisenaan; kysy suojelijan toimistolta, jos tapahtuma tai maastotyö koskee rekisteröityä kohdetta.
permits-hint-history-register-pl = Historian muistomerkki (Pomnik historii) on karttaalueella tai sen lähellä; historian muistomerkkejä koskeva osa kertoo, mikä. Puolan tasavallan presidentti voi asetuksella julistaa rekisteriin merkityn kiinteän muistomerkin tai erityisen arvokkaan kulttuuripuiston Pomnik historii -kohteeksi (muistomerkkien suojelulain, ustawa o ochronie zabytków, 15 artiklan 1 momentti). Se on siis jo rekisterin tai kulttuuripuiston suojeluksessa. Rekisteriin merkityn muistomerkin töihin tarvitaan vojevodikunnan muistomerkkien suojelijan (Wojewódzki Konserwator Zabytków) lupa (36 artiklan 1 momentti); kulttuuripuistossa kiellot ja rajoitukset määrää kunnanvaltuuston päätös (16 artiklan 2 momentti, 17 artiklan 1 momentti). Laki ei mainitse kävelyä, juoksua tai rastia sellaisenaan; kysy suojelijan toimistolta, jos tapahtuma tai maastotyö koskee muistomerkkiä.
permits-hint-gmina-nature-pl = { $gmina ->
        [none] Karttaalueella on luonnonmuistomerkki (pomnik przyrody), ekologinen alue (użytek ekologiczny), luonto- ja maisemakokonaisuus (zespół przyrodniczo-krajobrazowy) tai dokumentointikohde (stanowisko dokumentacyjne). Ne perustaa gminan valtuusto päätöksellä, josta käy ilmi, kuka valvoo kohdetta ja mitkä kiellot ovat voimassa (luonnonsuojelulain 44 artikla). Laki itse ei mainitse tapahtumalle tässä mitään lupaa. Pyydä päätös ja valvojan nimi sen gminan virastosta, jonka alueella kohde sijaitsee.
       *[other] Karttaalueella on luonnonmuistomerkki (pomnik przyrody), ekologinen alue (użytek ekologiczny), luonto- ja maisemakokonaisuus (zespół przyrodniczo-krajobrazowy) tai dokumentointikohde (stanowisko dokumentacyjne). Ne perustaa gminan valtuusto päätöksellä, josta käy ilmi, kuka valvoo kohdetta ja mitkä kiellot ovat voimassa (luonnonsuojelulain 44 artikla). Laki itse ei mainitse tapahtumalle tässä mitään lupaa. Pyydä päätös ja valvojan nimi { $count ->
        [one] sen gminan ({ $gmina }) virastosta, jonka alueella kohde sijaitsee
       *[other] näiden gminojen ({ $gmina }) virastoista, kukin omalla alueellaan sijaitsevien kohteiden osalta
    }.
    }
permits-hint-gmina-roads-pl = Tapahtuma, joka haittaa liikennettä tai käyttää katua erityisellä tavalla, tarvitsee tieliikennelain 65 artiklan mukaisen luvan, joka haetaan vähintään 30 päivää etukäteen. Luvan antaa tien liikennettä ohjaava viranomainen: starosta powiatin ja gminan teillä, pormestari powiat-oikeuksilla varustetussa kaupungissa, voivodikunnan marsalkka tai GDDKiA muilla teillä (artiklat 10 ja 65b). Gminan tien pitäjä { $count ->
        [one] on gminan ({ $gmina }) wójt
       *[other] on kunkin gminan ({ $gmina }) wójt
    }, burmistrz tai prezydent miasta; tienpitäjä päättää tiealueen käytöstä (yleisistä teistä annetun lain artiklat 19 ja 40). Nämä huomautukset eivät näytä teiden luokkaa.
permits-hint-gmina-land-pl = Jos karttaalueen maa kuuluu gminalle, esimerkiksi kaupunginpuisto, tori, koulun piha tai metsä, sitä hallinnoi wójt, burmistrz tai prezydent miasta, joka edustaa gminaa (gminan itsehallintolain artiklat 30 ja 31). Pyydä suostumus { $count ->
        [one] gminan ({ $gmina }) virastolta
       *[other] gminojen ({ $gmina }) virastoilta
    } ennen käyttöä. Nämä huomautukset eivät näytä, kuka maan omistaa.
permits-section-water-protection = Vedensuojelualueet ({ $source })
permits-section-forest-functions = Metsän toiminnot ja suojametsä ({ $source })
permits-none-water-protection = Lähteen { $source } mukaan kartta-alueella ei ole vedensuojelualuetta.
permits-none-forest-functions = Lähteen { $source } mukaan kartta-alueella ei ole virkistysmetsää, suojametsää, luonnonmetsää eikä suljettua metsäaluetta.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Virkistysmetsä (Erholungswald)
        [protected] Suojametsä (Bannwald tai Schutzwald)
        [natural] Luonnonmetsä tai ehdoton suojelualue (Naturwald, Totalreservat)
       *[closed] Suljettu metsäalue
    }
permits-hint-water-protection = { $inner ->
        [yes] Vedensuojelualueen "{ $name }" { $zone } on kartta-alueella. Vyöhykkeillä I ja II tapahtumat on yleensä suljettu pois. Kysy alemmalta vesiviranomaiselta (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
        [no] Vedensuojelualueen "{ $name }" { $zone } on kartta-alueella. Tapahtumat siellä ovat yleensä mahdollisia; alempi vesiviranomainen (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }) osaa kertoa, mitä asetus sallii.
       *[unknown] Vedensuojelualue "{ $name }" on kartta-alueella. Tiedot eivät kerro, mikä vyöhyke se on. Vyöhykkeillä I ja II tapahtumat on yleensä suljettu pois; kysy alemmalta vesiviranomaiselta (untere Wasserbehörde{ $authority ->
            [none] {""}
           *[other] {", "}{ $authority }
        }).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Virkistysmetsää (Erholungswald) on kartta-alueella{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Tapahtumista siellä kysy metsäviranomaiselta.
        [protected] Suojametsää (Bannwald tai Schutzwald) on kartta-alueella{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Kysy metsäviranomaiselta, ennen kuin suunnittelet tapahtumaa siellä.
        [natural] Luonnonmetsää tai ehdotonta suojelualuetta on kartta-alueella{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Polulta poikkeaminen on siellä usein kielletty; älä sijoita sinne rasteja kysymättä metsäviranomaiselta.
       *[closed] Metsäalue, jolle ei saa mennä, on kartta-alueella{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Pidä se radan ulkopuolella.
    }
permits-section-roads = Luokitellut tiet OpenStreetMapin mukaan
permits-none-roads = OpenStreetMapin mukaan kartta-alueen läpi ei kulje Autobahnia eikä Bundes-, Landes-, Staats- tai Kreisstraßea.
permits-road-line = { $class ->
        [autobahn] Autobahn (moottoritie)
        [bundesstrasse] Bundesstraße (liittovaltion tie)
        [landesstrasse] Landesstraße tai Staatsstraße (osavaltion tie)
       *[kreisstrasse] Kreisstraße (piirikunnan tie)
    }: { $refs }
permits-hint-road-class = { $class ->
        [autobahn] Autobahn { $refs } kulkee kartta-alueen läpi. Rata saa ylittää sen vain siltoja tai alikulkuja pitkin.
        [bundesstrasse] Bundesstraße { $refs } kulkee kartta-alueen läpi. Sulun tai ylityksen liikenteenohjaajin määrää tieliikenneviranomainen (Straßenverkehrsbehörde); tietä ylläpitää yleensä liittovaltio osavaltion tiehallinnon kautta, suuremmissa kaupungeissa usein kaupunki.
        [landesstrasse] Landes- tai Staatsstraße { $refs } kulkee kartta-alueen läpi. Sulun tai ylityksen liikenteenohjaajin määrää tieliikenneviranomainen (Straßenverkehrsbehörde); tietä ylläpitää yleensä osavaltio, suuremmissa kaupungeissa usein kaupunki.
       *[kreisstrasse] Kreisstraße { $refs } kulkee kartta-alueen läpi. Sulun tai ylityksen liikenteenohjaajin määrää tieliikenneviranomainen (Straßenverkehrsbehörde); tietä ylläpitää yleensä Kreis, suuremmissa kaupungeissa usein kaupunki.
    }
permits-hint-state-forest = { $office ->
        [none] Osavaltion avoimen datan mukaan { $share } kartta-alueesta on omistajan "{ $owner }" valtion metsää.
       *[other] Osavaltion avoimen datan mukaan { $share } kartta-alueesta on omistajan "{ $owner }" valtion metsää. Vastuussa: { $office }.
    }
