# Progress lines and warnings of the engine (ADR 0092, stage 3). English is the reference.
# Ids: progress- for progress lines, warning- for warnings, word- for names inside other
# messages. A warning gets its prefix from note-warning.

note-warning = varoitus: { $text }

warning-other-zone = kartta-alue on UTM-vyöhykkeellä { $zone } (EPSG:{ $epsg }); freemapper hakee kaiken uudelleen tässä vyöhykkeessä
warning-other-state = kartta-alue on nyt toisen osavaltion tiedoissa; freemapper hakee kaiken uudelleen
warning-beyond-coverage = osa kartta-alueesta on tuettujen osavaltioiden ulkopuolella. Se on ulkomailla, merellä tai osavaltiossa, jota freemapper ei vielä tue, kuten Thüringenissä. Osavaltioilla ei ole siellä tietoja, joten ortokuva voi olla valkoinen eikä ALKIS anna rakennuksia tai maankäyttöä
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: mittakaava 1:{ $scale } ei kuulu karttatyypin { $kind ->
        [schoolyard] koulunpiha
        [forest] metsä
       *[sprint] sprintti
    } mittakaavoihin ({ $allowed }); freemapper-print hylkää kartan
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: { $what ->
        [tree] puutason
        [NAS] NAS-tason
        [set] lisätason
       *[other] { $german }-tason
    } merkintä { $key } ohitettu: { $reason ->
        [name] ei ole pelkkä tiedostonimi
       *[unknown] ei freemapperin tuntemaa tasoa
    }

## Disk space

warning-disk-space-unreadable = kohteen { $path } vapaata levytilaa ei voi lukea ({ $error }), joten freemapper ei tarkista sitä

## Tiles

warning-tile-changed = { $product }-ruutu { $tile } muuttui lähteessä ({ $old } → { $new })
warning-bdom-tile-unavailable = ruudun { $tile } pintamalli (bDOM) ei ole saatavilla ({ $error }); ei nDSM-pohjaa siellä
warning-bdom-unavailable = pintamalli (bDOM) ei ole saatavilla ({ $error }); ei nDSM-pohjaa kohteille { $tiles }
progress-ndsm-from-bdom = ruudun { $tile } nDSM pintamallista (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] maastomalli
        [surface] pintamalli
       *[orthophoto] ortokuva
    } ({ $name })
progress-model-download = ladataan: { $model }
progress-model-read = luetaan: { $model }
warning-model-unavailable = { $model } ei saatavilla ({ $error }); ei nDSM-pohjaa
warning-model-unavailable-kept = { $model } ei saatavilla ({ $error }); säilytettiin päivänä { $fetched } haettu pintamalli
warning-model-tile-unavailable = { $model }, ruutu { $tile } ei saatavilla ({ $error }); ei nDSM-pohjaa siellä
warning-archive-passed-over = { $model }: arkistoa ei voitu lukea ({ $error }); se ohitettiin, ja kartta-alueen ruudut tulevat muista arkistoista
warning-archive-directory-kept = { $model }: arkistoa ei voitu lukea ({ $error }); käytetään sen aiemmasta ajosta säilytettyä hakemistoa, joten LVGL:n sittemmin korvaamaa ruutua ei huomata
warning-model-tiles-missing = { $model }: { $count ->
        [one] ruutu { $names } puuttuu
       *[other] ruudut { $names } puuttuvat
    }, alue: { $state ->
        [the-map-area] kartta-alue
        [hamburg] Hampuri
        [hesse] Hessen
        [north-rhine-westphalia] Nordrhein-Westfalen
        [schleswig-holstein] Schleswig-Holstein
        [mecklenburg-vorpommern] Mecklenburg-Vorpommern
        [lower-saxony] Ala-Saksi
        [bavaria] Baijeri
       *[other] { $state }
    }; pohjat jäävät siellä tyhjiksi
warning-no-surface-stored = pintamallia ei ole tallennettu; aja `freemapper fetch` verkossa nDSM-pohjaa varten
warning-surface-older-than-orthophoto = pintamalli (bDOM { $surface }) on ortokuvaa ({ $photo }) vanhempi; uusia rakennuksia voi puuttua nDSM:stä
progress-surface-kept = freemapper säilyttää päivänä { $fetched } haetun pintamallin nDSM-pohjaa varten
progress-surface-tiles-kept = freemapper säilyttää välimuistissa olevan pintamallin { $count ->
        [one] ruudulle
       *[other] ruuduille
    } { $names } nDSM-pohjaa varten
progress-orthophoto-unchanged = ortokuva ennallaan
progress-orthophoto-unchanged-dated = ortokuva ennallaan ({ $label } { $date })
progress-orthophoto-download = ladataan ortokuva: { $provider } { $service }
progress-orthophoto-in-leaf = ortokuva on lennetty { $date }, puut lehdessä; niiden latvukset peittävät alla olevan
progress-orthophoto-bare = ortokuva on lennetty { $date }, puut paljaina
warning-lgln-orthophoto-kept = ortokuva (LGLN WMS): { $reason }; säilytettiin tallennettu ortokuva ja sen lentopäivä { $date }
warning-lgln-orthophoto-undated = ortokuva (LGLN WMS): ei lentopäivää, { $reason }
word-stac-no-image = STAC ei luettele kuvaa kartta-alueen keskipisteessä
word-stac-no-answer = STAC-haku ei antanut käyttökelpoista vastausta ({ $error })
warning-downloadcenter-no-file = Downloadcenterissa ei ole tiedostoa kohteelle { $places }
warning-downloadcenter-no-file-empty = Downloadcenterissa ei ole tiedostoa kohteelle { $places }; { $count ->
        [one] ruutu
       *[other] ruudut
    } { $names } jäi tyhjäksi
progress-k-five-unchanged = K5-pohja ennallaan
progress-k-five-download = ladataan K5-kartta (1:5000) Geoportal Berlinin WMS-palvelusta
warning-k-five-unavailable = K5-karttapohja ei ole saatavilla ({ $detail }); ohitettu

## The state border

progress-beyond-border = kartta-alue ulottuu osavaltion rajan yli; sen takana pohjat ovat tyhjiä siellä, missä { $provider }-ortokuva ja korkeusmallit loppuvat. ALKIS ja OSM tulevat molemmista osavaltioista
progress-beyond-border-no-data = kartta-alue ulottuu osavaltion rajan yli; kohteelle { $tiles } ei ole { $provider }-tietoja, joten pohjat jäävät siellä tyhjiksi

## OSM

warning-osm-extract-missing-kept = OSM-otetta ei ole välimuistissa; säilytetään tallennetut OSM-ehdokkaat päivältä { $date }
progress-osm-extract-read = luetaan OSM-ote { $region } päivältä { $date }
progress-osm-candidates = { $count } OSM-ehdokaskohdetta

## Permit notes

progress-permits-fetch = haetaan lähteen { $source } tiedot lupahuomautuksia varten
progress-permits-osm = luetaan OSM-ote { $region } päivältä { $date } lupahuomautuksia varten
progress-permits-stored = luetaan lupahuomautusten tallennetut tiedot
warning-permits-source-unreachable = lähdettä { $source } ei voitu lukea, joten lupahuomautukset jättävät sen pois: { $error }
warning-permits-osm-no-extract = tälle kartta-alueelle ei ole OSM-otetta välimuistissa, joten lupahuomautukset jättävät OSM:n pois; `freemapper fetch` lataa sen

## ALKIS

warning-alkis-coverage-unknown = { $cadastre }-rakennusten kattavuus tuntematon; aja `freemapper fetch` verkossa tallentaaksesi sen
progress-alkis-buildings-download = ladataan { $cadastre }-rakennuksia
progress-alkis-buildings = { $count } { $cadastre }-rakennusta
warning-alkis-no-buildings = kartta-alueelta ei löytynyt { $cadastre }-rakennuksia
progress-alkis-skipped = ohitettiin { $count } { $cadastre }-kohdetta: { $list }
warning-alkis-buildings-changed = { $cadastre }-rakennukset muuttuivat lähteessä ({ $changes })
progress-alkis-no-land-use-stored = { $cadastre }-maankäyttöä ei ole tallennettu; aja `freemapper fetch` verkossa kiellettyjen alueiden ehdokkaita varten
progress-alkis-land-use-download = ladataan { $cadastre }-maankäyttöä
progress-alkis-land-use-parts = { $count } { $cadastre }-maankäytön osaa
warning-alkis-land-use-kept = { $cadastre }-maankäyttö ei ole saatavilla ({ $error }); säilytetään tallennettu maankäyttö
warning-alkis-land-use-removed = { $cadastre }-maankäyttö ei ole saatavilla ({ $error }); tallennettu maankäyttö ei enää kata kartta-aluetta, joten freemapper poisti sen
warning-alkis-land-use-none = { $cadastre }-maankäyttö ei ole saatavilla ({ $error }); ei kiellettyjen alueiden ehdokkaita

## The Berlin layers

progress-berlin-none-stored = Berliinin ALKIS-rakenteita ei ole tallennettu; aja `freemapper fetch` verkossa portaita, muureja, puita ja katoksia varten
progress-berlin-removing = poistetaan tallennetut Berliinin tasot: kartta-alue ei ole enää lähellä Berliiniä
progress-berlin-download = ladataan Berliinin ALKIS-rakenteita, puurekisteriä, katukartoitusta ja pistetasoja
progress-berlin-features = { $count } Berliinin { $file }-kohdetta
warning-berlin-layer-kept = Berliinin { $file } ei saatavilla ({ $detail }); säilytetään tallennettu taso
warning-berlin-layer-skipped = Berliinin { $file } ei saatavilla ({ $detail }); taso ohitettu
progress-feinkartierung-none-stored = Hampurin Feinkartierungia ei ole tallennettu; aja `freemapper fetch` verkossa kaiteita, muureja, portaita ja katukalusteita varten
progress-feinkartierung-removing = poistetaan tallennettu Hampurin Feinkartierung: kartta-alue ei ole enää Hampurissa
progress-feinkartierung-download = ladataan Hampurin Feinkartierung (katutila ja yleinen vihreä)
progress-feinkartierung-features = { $count } Feinkartierung-kohdetta ({ $file })
warning-feinkartierung-layer-kept = Feinkartierung { $file } ei saatavilla ({ $detail }); säilytetään tallennettu taso
warning-feinkartierung-layer-skipped = Feinkartierung { $file } ei saatavilla ({ $detail }); taso ohitettu
error-extra-layer-unknown = ei ole kansion { $folder } taso
warning-berlin-buildings-missing = tälle alueelle Berliinissä ei ole tallennettu ALKIS-rakennuksia; aja `freemapper fetch` hakeaksesi ne Geoportal Berlinistä

## The full ALKIS layers (NAS)

progress-nas-none-stored = täysiä ALKIS-rakenteita ei ole tallennettu; aja `freemapper fetch` verkossa muureja, aitoja, pensasaitoja ja siltoja varten
progress-nas-removing = poistetaan tallennetut täydet ALKIS-tasot: kartta-alue ei ole enää osavaltiossa, jossa on NAS-palvelu
warning-nas-folder-outside = täydet ALKIS-tasot ohitettu (kansio { $folder } johtaa projektikansion ulkopuolelle)
progress-nas-download = ladataan täydet ALKIS-rakenteet (NAS)
progress-nas-features = { $count } ALKIS NAS -kohdetta ({ $file })
progress-nas-skipped = ohitettiin { $count } ALKIS NAS -kohdetta ({ $file }): { $list }
warning-nas-layer-kept = ALKIS NAS { $file } ei saatavilla ({ $detail }); säilytetään tallennettu taso
warning-nas-layer-skipped = ALKIS NAS { $file } ei saatavilla ({ $detail }); taso ohitettu

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = ATKIS Basis-DLM:n kohteita ei ole tallennettu; aja `freemapper fetch` verkossa lähteitä, luolia, torneja, muistomerkkejä ja suihkulähteitä varten
progress-dlm-removing = poistetaan tallennetut ATKIS Basis-DLM -tasot: kartta-alue ei ole enää osavaltiossa, joka tarjoaa ne
warning-dlm-folder-outside = ATKIS Basis-DLM -tasot ohitettu (kansio { $folder } johtaa projektikansion ulkopuolelle)
progress-dlm-download = ladataan ATKIS Basis-DLM:n kohteita
progress-dlm-features = { $count } ATKIS Basis-DLM -kohdetta ({ $file })
progress-dlm-skipped = ohitettiin { $count } ATKIS Basis-DLM -kohdetta ({ $file }): { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file } ei saatavilla ({ $detail }); säilytetään tallennettu taso
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file } ei saatavilla ({ $detail }); taso ohitettu

## Tree cadastres

progress-trees-none-stored = kaupungin puurekisteriä ei ole tallennettu; aja `freemapper fetch` verkossa yksittäisiä puita varten
progress-trees-removing = poistetaan tallennetut puurekisterit: kartta-alue ei ole enää kaupungissa, jolla sellainen on
warning-trees-folder-outside = kaupunkien puurekisterit ohitettu (kansio { $folder } johtaa projektikansion ulkopuolelle)
progress-trees-download = ladataan kaupunkien puurekistereitä
progress-trees-count = { $count } puuta ({ $id })
progress-trees-skipped = ohitettiin { $count } puuta ({ $id }): { $list }
warning-trees-kept = puurekisteri { $id } ei saatavilla ({ $detail }); säilytetään tallennettu
warning-trees-skipped = puurekisteri { $id } ei saatavilla ({ $detail }); puut ohitettu

## Deriving the templates

progress-writing = kirjoitetaan { $path }
progress-contours-steep = korkeuskäyrät { $interval } m välein: maasto on niin jyrkkää, että { $default } m välein vedetyt käyrät sulautuisivat yhteen; projektin korkeuskäyrien väli { $default } m pitää ne erillään (`--contour-interval { $default }`)
warning-osm-only-no-terrain = tälle alueelle ei vielä ole maastotietoja: kartta tehdään pelkästä OpenStreetMapista, ilman korkeuskäyriä, pohjia ja rakennuksia
progress-no-ndsm-recorded = nDSM:ää ei ole kirjattu; aja `freemapper fetch` nDSM-pohjaa varten
warning-ndsm-tile-invalid = virheellinen nDSM-ruutu { $tile } tiedostossa freemapper.toml; ohitettu
warning-ndsm-not-cached = nDSM { $tile } ei ole välimuistissa; aja `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } ei ole luettavissa ({ $error }); poista { $path } ja aja `freemapper fetch`
warning-surface-unusable = pintamalli ei ole käyttökelpoinen ({ $error }); aja `freemapper fetch`

## Downloads and the cache

progress-downloading = ladataan { $url }
progress-downloaded = ladattu { $megabytes } MB
progress-downloaded-of = ladattu { $megabytes } MB / { $total } MB
progress-tile-index-download = ladataan ruutuindeksi: { $product }
progress-tile-cached = välimuistissa { $tile }
progress-surface-blocks = luetaan { $count } / { $total } lohkoa pintamallin ruudusta { $tile } ({ $megabytes } MB)
progress-surface-whole = pintamallin ruutu { $tile } ladataan kokonaisena: { $reason }
warning-tile-cached-older = käytetään välimuistissa olevaa { $product }-ruutua { $tile } päivältä { $published } (uudempaa { $newer } ei ladattu)
warning-ndsm-no-cached = ruudulle { $tile } ei ole nDSM:ää välimuistissa
warning-ndsm-tile-unavailable-cached = nDSM-ruutu { $tile } ei saatavilla ({ $reason }); käytetään välimuistissa olevaa tiedostoa
warning-ndsm-tile-unavailable-skipped = nDSM-ruutu { $tile } ei saatavilla ({ $reason }); ohitettu
warning-file-check-failed = ei voitu tarkistaa, onko { $file } yhä ajan tasalla ({ $reason }); käytetään välimuistissa olevaa tiedostoa ja tämän palvelimen muita välimuistissa olevia tiedostoja kysymättä
warning-ldbv-no-buildings-file = ruudulle { $tile } ei ole rakennustiedostoa palvelimella; kartalta voi puuttua rakennuksia siellä
progress-ndsm-not-published = ruudulle { $tile } ei ole julkaistu nDSM-pistepilveä
progress-ndsm-cached = nDSM { $tile } välimuistissa
progress-point-cloud-not-published = ruudulle { $tile } ei ole julkaistu laserpistepilveä
progress-point-cloud-cached = pistepilvi { $tile } välimuistissa
warning-point-cloud-no-cached = ruudulle { $tile } ei ole pistepilveä välimuistissa; ei kasvillisuusehdokkaita siellä
warning-point-cloud-unavailable-cached = pistepilvi { $tile } ei saatavilla ({ $reason }); käytetään välimuistissa olevaa tiedostoa
warning-point-cloud-unavailable-skipped = pistepilvi { $tile } ei saatavilla ({ $reason }); ei kasvillisuusehdokkaita siellä
warning-point-clouds-unavailable = laserpistepilvet eivät ole saatavilla ({ $reason }); kasvillisuusehdokkaat käyttävät välimuistissa olevia ruutuja, jos niitä on
progress-no-point-clouds = kasvillisuudelle ei ole täällä vielä laserpistepilviä: osavaltio ei julkaise sellaisia, jotka freemapper osaa lukea, tai ne eivät ole kartan koordinaattijärjestelmässä
warning-point-clouds-not-fetched = kasvillisuus on valittu, mutta pistepilviä ei ole ladattu; aja `freemapper fetch` ladataksesi ne
warning-point-cloud-unreadable = pistepilvi { $tile } ei ole luettavissa ({ $error }); poista { $path } ja aja `freemapper fetch`
warning-osm-nrw-nearest = alueen 100 metrin sisällä ei ole Nordrhein-Westfalenin piiriotetta; luetaan lähin, { $district }
warning-osm-disk-full-cached = levy on täynnä, joten freemapper käyttää välimuistissa olevaa OSM-otetta päivältä { $date }. Vapauta tilaa ja aja `freemapper fetch` uudelleen saadaksesi uudemman
warning-osm-geofabrik-unavailable = Geofabrik ei saatavilla ({ $error }); käytetään välimuistissa olevaa OSM-otetta päivältä { $date }
progress-osm-extract-cached = OSM-ote { $region } päivältä { $date } välimuistissa
progress-osm-extract-download = ladataan { $url } (noin { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = ladataan bDOM-ruutu { $tile } ({ $megabytes } MB, { $index } / { $total })
progress-sh-dgm-not-in-atom = DGM1-ruutu { $tile } ei ole INSPIRE ATOM -hakemistossa; ladataan sen XYZ ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = DGM1-ruutu { $tile } INSPIRE ATOM -hakemistosta ei ole luettavissa ({ $error }); ladataan sen XYZ ({ $megabytes } MB)
progress-sh-tile = ruutu { $tile }: { $text }
progress-sh-download-again = ruutu { $tile }: lataus ei ole luettavissa ({ $message }); ladataan se vielä kerran
warning-sh-portal-no-date = Downloadportal ei anna päivämäärää ruudulle { $tile } ({ $model })
warning-sh-dgm-xyz-used = DGM1-ruutu { $tile } ei ole INSPIRE ATOM -hakemistossa; käytettiin sen XYZ:ää Downloadportalista
warning-sh-dgm-xyz-used-unreadable = DGM1-ruutu { $tile } INSPIRE ATOM -hakemistosta ei ole luettavissa ({ $error }); käytettiin sen XYZ:ää Downloadportalista

## Tile lists and dates of the height models

warning-model-tile-cells-empty = { $model }, ruutu { $tile }: { $count } kartta-alueen solulla ei ole korkeutta
warning-centre-tile-missing = { $model }-ruutu { $tile } kartta-alueen keskipisteessä puuttuu, joten päivämäärää ei ole
warning-centre-tile-unlisted = ruutuluettelo ei nimeä { $model }-ruutua { $tile } kartta-alueen keskipisteessä, joten päivämäärää ei ole
warning-list-unreadable-kept = mallin { $model } ruutuluetteloa ei voitu lukea ({ $error }); käytettiin aiemmasta ajosta säilytettyä
warning-portal-list-unreadable-kept = Downloadportalin ruutuluetteloa mallille { $model } ei voitu lukea ({ $error }); käytettiin aiemmasta ajosta säilytettyä
warning-surface-much-older = { $model } on vuodelta { $surface }, ortokuva vuodelta { $flight }. nDSM:stä voi puuttua uudempia rakennuksia ja puita.

## Rhineland-Palatinate

warning-rp-list-features-unusable = { $count } ruutuluettelon kohteella ei ole käyttökelpoista ruutuavainta; ohitettu
warning-rp-list-unreadable-kept = LVermGeo RP:n ruutuluetteloa ei voitu lukea ({ $error }); käytettiin aiemmasta ajosta säilytettyä
warning-rp-metalink-not-kept = LVermGeo RP:n Metalinkiä ei säilytetty: { $error }
warning-rp-metalink-unreadable-kept = LVermGeo RP:n Metalinkiä mallille { $model } ei voitu lukea ({ $detail }); käytettiin aiemmasta ajosta säilytettyä
warning-rp-metalink-unreadable = LVermGeo RP:n Metalinkiä mallille { $model } ei voitu lukea ({ $detail }), joten ruudut, joille sen ruutuluettelo ei anna lentovuotta, jäävät ilman korkeuksia
warning-rp-tile-without-name = { $model }, ruutu { $tile }: tällä 1 km:n neliöllä ei ole korkeuksia kartassa. LVermGeo RP:n ruutuluettelo (Laserbefliegung) { $note }, joten tiedoston nimi ei ole tiedossa{ $either ->
        [metalink] {", "}eikä sen Metalink nimeä sille tiedostoa myöskään
       *[none] {""}
    }
warning-rp-centre-no-date = { $model }-ruudulla { $tile } ei ole päivämäärää: LVermGeo RP:n ruutuluettelo (Laserbefliegung) { $note }
word-rp-list-neither = ei anna sille lentovuotta eikä lentopäiviä
word-rp-list-no-days = ei anna sille lentopäiviä
word-rp-list-no-year = ei anna sille lentovuotta
word-rp-list-days-unreadable = antaa sille lentopäivät { $days }, joita ei voi lukea
word-rp-list-disagree = antaa sille lentopäivät { $days } ja lentovuoden { $year }, jotka eivät täsmää samaan vuoteen

## Hesse

warning-hvbg-undated = metatietoluettelossa ei ole luettavaa Bildflugdatumia ruuduille { $tiles }; niiden projektitiedoissa ei ole lentopäivää
warning-hvbg-undated-more = metatietoluettelossa ei ole luettavaa Bildflugdatumia ruuduille { $tiles } ja { $more } muulle; niiden projektitiedoissa ei ole lentopäivää
warning-hvbg-list-unreadable-older = uutta HVBG:n metatietoluetteloa { $name } ei voitu lukea ({ $detail }); käytetään vanhempaa luetteloa { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: ei päivämäärää, STAC ei luettele ruutua kartta-alueen keskipisteessä
warning-lgln-no-date = { $model }: ei päivämäärää ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; tallennettu malli säilyttää kirjatun päivämääränsä
warning-mv-cells-empty = { $model }: { $count } Mecklenburg-Vorpommernin ruutujen sisällä olevalla solulla ei ole korkeutta
warning-mv-centre-no-date = { $model }: ei päivämäärää keskiruudulle { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = { $count } mallin { $model } ruutuluettelon riviltä puuttuu käyttökelpoinen ruudun nimi; ohitettu
warning-nrw-list-rows-undated = { $count } mallin { $model } ruutuluettelon rivillä ei ole luettavaa päivämäärää
warning-nrw-list-older = palvelimen ruutuluettelo mallille { $model } on päivältä { $server }, vanhempi kuin säilytetty päivältä { $kept }; käytettiin säilytettyä
warning-nrw-list-no-date = ruutuluettelo ei anna päivämäärää mallin { $model } ruudulle { $tile }

## After `init`

note-scale-outside-norm = huomautus: 1:5000 ei kuulu sprinttinormiin; freemapper-print varoittaa siitä ja symbolit pysyvät 100 %:ssa

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } ei ole OSM-tunnus, kuten way/992423119
error-place-invalid-answer = virheellinen vastaus paikkahausta: { $detail }
error-place-not-found = { $id } ei löydy. Nominatim luettelee vain tunnisteilla varustetut OSM-kohteet, joten monikulmion tunnisteeton ulkoreuna ei ole siellä. Käytä sen sijaan monikulmiorelaatiota (--osm relation/<id>) tai tallenna ääriviiva GeoJSON-tiedostoksi ja käytä valitsinta --geojson
error-place-is-a-point = { $id } on piste, ei alue; valitse sen alue (polku tai relaatio)
error-place-no-such-part = { $id }: { $count ->
        [one] 1 osa
       *[other] { $count } osaa
    }; osaa { $part } ei ole
error-place-several-parts = { $id } on { $count } erillisen osan alue ({ $sizes }). Kartta-alue on yksi monikulmio: valitse osa valitsimella --part <n> (1 on suurin) tai tallenna haluamasi ääriviiva GeoJSON-tiedostoksi ja käytä valitsinta --geojson
error-layer-not-a-world-file = { $path }: ei ole world-tiedosto
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = tasolle { $layer } ei ole esikatselua; tasot, joilla on esikatselu: { $names }
error-layer-no-ndsm-published = tämän kartta-alueen ruuduille ei ole julkaistu nDSM:ää, joten ndsm-pohjaa ei ole
error-layer-not-made = { $layer } ei ole vielä tehty; aja { $step ->
        [fetch] freemapper fetch tai run (MCP: start_pipeline); vain Berliini
        [export] freemapper export tai run (MCP: start_pipeline)
       *[derive] freemapper derive tai run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; OCAD-kartta ei saa siitä taustaa
warning-map-preview-failed = kartta on kirjoitettu, mutta sen esikatselu puuttuu: { $error }
warning-training-no-tree-cover = harjoituskartta säilyttää puistojen avoimen maan kokonaisena: puuston peitto (data/tree-cover.tif) puuttuu; aja `freemapper derive` uudelleen
