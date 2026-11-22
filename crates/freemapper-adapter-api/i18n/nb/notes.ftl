# Fremdriftslinjer og advarsler fra motoren (ADR 0092, trinn 3). Engelsk er referansen.
# Ids: progress- for fremdriftslinjer, warning- for advarsler, word- for navn inni andre
# meldinger. En advarsel får prefikset sitt fra note-warning.

note-warning = advarsel: { $text }

warning-other-zone = kartområdet ligger i UTM-sone { $zone } (EPSG:{ $epsg }); freemapper henter alt på nytt i den sonen
warning-other-state = kartområdet ligger nå i en annen delstats data; freemapper henter alt på nytt
warning-beyond-coverage = en del av kartområdet ligger utenfor de støttede delstatene. Den ligger i utlandet, på havet eller i en delstat freemapper ennå ikke støtter, for eksempel Thüringen. Delstatene har ingen data der, så ortofotoet kan være hvitt og ALKIS gir ingen bygninger eller arealbruk
# A warning whose text is an error.
warning-error = { $error }

## The project file

warning-scale-not-allowed = freemapper.toml: målestokk 1:{ $scale } er ingen av målestokkene for { $kind ->
        [schoolyard] skolegårdskart
        [forest] skogskart
       *[sprint] sprintkart
    } ({ $allowed }); freemapper-print vil avvise kartet
# `$german` is the name of a set's state in the German text; the English text reads it only
# so that both catalogs name the same variables.
warning-layer-entry-ignored = freemapper.toml: ignorerte oppføringen { $key } i { $what ->
        [tree] trelagene
        [NAS] NAS-lagene
        [set] tilleggslagene
       *[other] { $german }-lagene
    }: { $reason ->
        [name] ikke et enkelt filnavn
       *[unknown] ikke et lag freemapper kjenner
    }

## Disk space

warning-disk-space-unreadable = kan ikke lese ledig diskplass for { $path } ({ $error }), så freemapper sjekker den ikke

## Tiles

warning-tile-changed = { $product }-flis { $tile } er endret hos leverandøren ({ $old } → { $new })
warning-bdom-tile-unavailable = overflatemodell (bDOM) for flis { $tile } ikke tilgjengelig ({ $error }); ingen nDSM-mal der
warning-bdom-unavailable = overflatemodell (bDOM) ikke tilgjengelig ({ $error }); ingen nDSM-mal for { $tiles }
progress-ndsm-from-bdom = nDSM for flis { $tile } fra overflatemodellen (bDOM)

## Height models and orthophotos

word-model = { $kind ->
        [terrain] terrengmodell
        [surface] overflatemodell
       *[orthophoto] ortofoto
    } ({ $name })
progress-model-download = laster ned { $model }
progress-model-read = leser { $model }
warning-model-unavailable = { $model } ikke tilgjengelig ({ $error }); ingen nDSM-mal
warning-model-unavailable-kept = { $model } ikke tilgjengelig ({ $error }); beholdt overflatemodellen som ble hentet { $fetched }
warning-model-tile-unavailable = { $model } for flis { $tile } ikke tilgjengelig ({ $error }); ingen nDSM-mal der
warning-archive-passed-over = { $model }: et arkiv kunne ikke leses ({ $error }); det ble hoppet over, og flisene i kartområdet kommer fra de andre arkivene
warning-archive-directory-kept = { $model }: et arkiv kunne ikke leses ({ $error }); katalogen fra en tidligere kjøring brukes, så en flis som LVGL har erstattet siden, blir ikke oppdaget
warning-model-tiles-missing = { $model } har ingen { $count ->
        [one] flis
       *[other] fliser
    } { $names } innenfor { $state ->
        [the-map-area] kartområdet
        [hamburg] Hamburg
        [hesse] Hessen
        [north-rhine-westphalia] Nordrhein-Westfalen
        [schleswig-holstein] Schleswig-Holstein
        [mecklenburg-vorpommern] Mecklenburg-Vorpommern
        [lower-saxony] Niedersachsen
        [bavaria] Bayern
       *[other] { $state }
    }; malene forblir tomme der
warning-no-surface-stored = ingen overflatemodell lagret; kjør `freemapper fetch` tilkoblet for nDSM-malen
warning-surface-older-than-orthophoto = overflatemodellen (bDOM { $surface }) er eldre enn ortofotoet ({ $photo }); nye bygninger kan mangle i nDSM
progress-surface-kept = freemapper beholder overflatemodellen som ble hentet { $fetched } for nDSM-malen
progress-surface-tiles-kept = freemapper beholder den bufrede overflatemodellen for { $count ->
        [one] flis
       *[other] fliser
    } { $names } for nDSM-malen
progress-orthophoto-unchanged = ortofoto uendret
progress-orthophoto-unchanged-dated = ortofoto uendret ({ $label } { $date })
progress-orthophoto-download = laster ned ortofotoet fra { $provider } { $service }
progress-orthophoto-in-leaf = ortofotoet ble fløyet { $date }, med trærne i løv; kronene skjuler det som ligger under dem
progress-orthophoto-bare = ortofotoet ble fløyet { $date }, med bare trær
warning-lgln-orthophoto-kept = ortofoto (LGLN WMS): { $reason }; beholdt det lagrede ortofotoet og flydatoen { $date }
warning-lgln-orthophoto-undated = ortofoto (LGLN WMS): ingen flydato, { $reason }
word-stac-no-image = STAC lister ikke noe bilde i sentrum av kartområdet
word-stac-no-answer = STAC-søket ga ikke noe brukbart svar ({ $error })
warning-downloadcenter-no-file = Downloadcenter har ingen fil for { $places }
warning-downloadcenter-no-file-empty = Downloadcenter har ingen fil for { $places }; { $count ->
        [one] flis
       *[other] fliser
    } { $names } ble stående tomme
progress-k-five-unchanged = K5-mal uendret
progress-k-five-download = laster ned K5-kartet (1:5000) fra Geoportal Berlin WMS
warning-k-five-unavailable = K5-kartmalen er ikke tilgjengelig ({ $detail }); hoppet over

## The state border

progress-beyond-border = kartområdet rekker over delstatsgrensen; på den andre siden er malene tomme der ortofotoet og høydemodellene fra { $provider } slutter. ALKIS og OSM kommer fra begge delstatene
progress-beyond-border-no-data = kartområdet rekker over delstatsgrensen; ingen data fra { $provider } for { $tiles }, så malene forblir tomme der

## OSM

warning-osm-extract-missing-kept = ikke noe OSM-utdrag i hurtigbufferet; beholder de lagrede OSM-kandidatene fra { $date }
progress-osm-extract-read = leser OSM-utdraget { $region } fra { $date }
progress-osm-candidates = { $count } OSM-kandidatobjekter

## Permit notes

progress-permits-fetch = henter { $source }-data til tillatelsesnotatene
progress-permits-osm = leser OSM-utdraget { $region } fra { $date } til tillatelsesnotatene
progress-permits-stored = leser de lagrede dataene til tillatelsesnotatene
warning-permits-source-unreachable = { $source } kunne ikke leses, så tillatelsesnotatene utelater den: { $error }
warning-permits-osm-no-extract = ikke noe OSM-utdrag i hurtigbufferet for dette kartområdet, så tillatelsesnotatene utelater OSM; `freemapper fetch` laster det ned

## ALKIS

warning-alkis-coverage-unknown = dekningen av { $cadastre }-bygninger er ukjent; kjør `freemapper fetch` tilkoblet for å registrere den
progress-alkis-buildings-download = laster ned { $cadastre }-bygninger
progress-alkis-buildings = { $count } { $cadastre }-bygninger
warning-alkis-no-buildings = ingen { $cadastre }-bygninger funnet i kartområdet
progress-alkis-skipped = hoppet over { $count } { $cadastre }-objekter: { $list }
warning-alkis-buildings-changed = { $cadastre }-bygninger er endret hos leverandøren ({ $changes })
progress-alkis-no-land-use-stored = ingen { $cadastre }-arealbruk lagret; kjør `freemapper fetch` tilkoblet for forbudsområdekandidater
progress-alkis-land-use-download = laster ned { $cadastre }-arealbruk
progress-alkis-land-use-parts = { $count } { $cadastre }-arealbruksdeler
warning-alkis-land-use-kept = { $cadastre }-arealbruk ikke tilgjengelig ({ $error }); beholder den lagrede arealbruken
warning-alkis-land-use-removed = { $cadastre }-arealbruk ikke tilgjengelig ({ $error }); den lagrede arealbruken dekker ikke lenger kartområdet, så freemapper fjernet den
warning-alkis-land-use-none = { $cadastre }-arealbruk ikke tilgjengelig ({ $error }); ingen forbudsområdekandidater
progress-bdot-filled = { $count } BDOT10k-objekter ({ $what }) lagt til der EGiB ikke gir noe ({ $powiat })
warning-bdot-failed = BDOT10k kunne ikke fylle hullet i EGiB ({ $what }): { $error }
warning-bdot-instead = EGiB er ikke tilgjengelig ({ $error }); BDOT10k brukes i stedet ({ $what }, { $powiat })

## The Berlin layers

progress-berlin-none-stored = ingen ALKIS-strukturer fra Berlin lagret; kjør `freemapper fetch` tilkoblet for trapper, murer, trær og kroner
progress-berlin-removing = fjerner de lagrede Berlin-lagene: kartområdet ligger ikke lenger nær Berlin
progress-berlin-download = laster ned Berlins ALKIS-strukturer, trematrikkel, gatekartlegging og punktlag
progress-berlin-features = { $count } Berlin { $file }-objekter
warning-berlin-layer-kept = Berlin { $file } ikke tilgjengelig ({ $detail }); beholder det lagrede laget
warning-berlin-layer-skipped = Berlin { $file } ikke tilgjengelig ({ $detail }); laget hoppet over
progress-feinkartierung-none-stored = ingen Hamburg Feinkartierung lagret; kjør `freemapper fetch` tilkoblet for rekkverk, murer, trapper og gatemøbler
progress-feinkartierung-removing = fjerner den lagrede Hamburg Feinkartierung: kartområdet ligger ikke lenger i Hamburg
progress-feinkartierung-download = laster ned Hamburg Feinkartierung (gaterom og offentlig grønt)
progress-feinkartierung-features = { $count } Feinkartierung { $file }-objekter
warning-feinkartierung-layer-kept = Feinkartierung { $file } ikke tilgjengelig ({ $detail }); beholder det lagrede laget
warning-feinkartierung-layer-skipped = Feinkartierung { $file } ikke tilgjengelig ({ $detail }); laget hoppet over
progress-bdot-roads-none-stored = ingen BDOT10k-veier og -stier lagret; kjør `freemapper fetch` på nett for å hente veiene og stiene fra registeret
progress-bdot-roads-removing = fjerner de lagrede BDOT10k-veiene og -stiene: kartområdet ligger ikke lenger i Polen
progress-bdot-roads-download = laster ned BDOT10k-veier og -stier
progress-bdot-roads-features = { $count } BDOT10k-objekter i { $file }
warning-bdot-roads-layer-kept = BDOT10k { $file } er utilgjengelig ({ $detail }); beholder det lagrede laget
warning-bdot-roads-layer-skipped = BDOT10k { $file } er utilgjengelig ({ $detail }); laget hoppes over
error-extra-layer-unknown = ikke et lag i { $folder }
warning-berlin-buildings-missing = ingen ALKIS-bygninger lagret for dette området i Berlin; kjør `freemapper fetch` for å hente dem fra Geoportal Berlin

## The full ALKIS layers (NAS)

progress-nas-none-stored = ingen fulle ALKIS-strukturer lagret; kjør `freemapper fetch` tilkoblet for murer, gjerder, hekker og bruer
progress-nas-removing = fjerner de lagrede fulle ALKIS-lagene: kartområdet ligger ikke lenger i en delstat med NAS-tjeneste
warning-nas-folder-outside = fulle ALKIS-lag hoppet over ({ $folder } fører ut av prosjektmappen)
progress-nas-download = laster ned de fulle ALKIS-strukturene (NAS)
progress-nas-features = { $count } ALKIS NAS { $file }-objekter
progress-nas-skipped = hoppet over { $count } ALKIS NAS { $file }-objekter: { $list }
warning-nas-layer-kept = ALKIS NAS { $file } ikke tilgjengelig ({ $detail }); beholder det lagrede laget
warning-nas-layer-skipped = ALKIS NAS { $file } ikke tilgjengelig ({ $detail }); laget hoppet over

## The objects of a landscape model (ATKIS Basis-DLM)

progress-dlm-none-stored = ingen objekter fra ATKIS Basis-DLM lagret; kjør `freemapper fetch` tilkoblet for kilder, huler, tårn, minnesmerker og fontener
progress-dlm-removing = fjerner de lagrede lagene fra ATKIS Basis-DLM: kartområdet ligger ikke lenger i en delstat som tilbyr dem
warning-dlm-folder-outside = lag fra ATKIS Basis-DLM hoppet over ({ $folder } fører ut av prosjektmappen)
progress-dlm-download = laster ned objektene fra ATKIS Basis-DLM
progress-dlm-features = { $count } ATKIS Basis-DLM { $file }-objekter
progress-dlm-skipped = hoppet over { $count } ATKIS Basis-DLM { $file }-objekter: { $list }
warning-dlm-layer-kept = ATKIS Basis-DLM { $file } ikke tilgjengelig ({ $detail }); beholder det lagrede laget
warning-dlm-layer-skipped = ATKIS Basis-DLM { $file } ikke tilgjengelig ({ $detail }); laget hoppet over

## Tree cadastres

progress-trees-none-stored = ingen bytrematrikkel lagret; kjør `freemapper fetch` tilkoblet for enkelttrærne
progress-trees-removing = fjerner de lagrede trematrikklene: kartområdet ligger ikke lenger i en by som har en
warning-trees-folder-outside = bytrematrikler hoppet over ({ $folder } fører ut av prosjektmappen)
progress-trees-download = laster ned bytrematrikklene
progress-trees-count = { $count } { $id }-trær
progress-trees-skipped = hoppet over { $count } { $id }-trær: { $list }
warning-trees-kept = trematrikkelen { $id } ikke tilgjengelig ({ $detail }); beholder den lagrede
warning-trees-skipped = trematrikkelen { $id } ikke tilgjengelig ({ $detail }); trær hoppet over

## Deriving the templates

progress-writing = skriver { $path }
progress-contours-steep = høydekurver hver { $interval } m: terrenget er så bratt at høydekurver hver { $default } m ville flyte sammen; en ekvidistanse på { $default } m i prosjektet beholder dem (`--contour-interval { $default }`)
warning-osm-only-no-terrain = ingen terrengdata for denne regionen ennå: kartet lages av OpenStreetMap alene, uten høydekurver, maler og bygninger
progress-no-ndsm-recorded = ingen nDSM registrert; kjør `freemapper fetch` for nDSM-malen
warning-ndsm-tile-invalid = ugyldig nDSM-flis { $tile } i freemapper.toml; hoppet over
warning-ndsm-not-cached = nDSM { $tile } ikke i hurtigbufferet; kjør `freemapper fetch`
warning-ndsm-unreadable = nDSM { $tile } kan ikke leses ({ $error }); slett { $path } og kjør `freemapper fetch`
warning-surface-unusable = overflatemodellen er ubrukelig ({ $error }); kjør `freemapper fetch`

## Downloads and the cache

progress-downloading = laster ned { $url }
progress-retry-wait = tjenesten er opptatt; prøver igjen om { $seconds } s
progress-downloaded = lastet ned { $megabytes } MB
progress-downloaded-of = lastet ned { $megabytes } MB av { $total } MB
progress-tile-index-download = laster ned flisindeksen for { $product }
progress-tile-cached = bufret { $tile }
progress-surface-blocks = leser { $count } av { $total } blokker i flisen { $tile } av overflatemodellen ({ $megabytes } MB)
progress-surface-whole = flisen { $tile } av overflatemodellen lastes ned hel: { $reason }
warning-tile-cached-older = bruker den bufrede { $product }-flisen { $tile } fra { $published } (nyere { $newer } ikke lastet ned)
warning-ndsm-no-cached = ingen bufret nDSM for flis { $tile }
warning-ndsm-tile-unavailable-cached = nDSM-flis { $tile } ikke tilgjengelig ({ $reason }); bruker den bufrede filen
warning-ndsm-tile-unavailable-skipped = nDSM-flis { $tile } ikke tilgjengelig ({ $reason }); hoppet over
warning-file-check-failed = kunne ikke sjekke om { $file } fortsatt er aktuell ({ $reason }); bruker den bufrede filen, og de andre bufrede filene fra denne tjeneren uten å spørre
warning-ldbv-no-buildings-file = ingen bygningsfil for flis { $tile } på tjeneren; kartet kan mangle bygninger der
progress-ndsm-not-published = ingen nDSM-punktsky publisert for flis { $tile }
progress-ndsm-cached = bufret nDSM { $tile }
progress-point-cloud-not-published = ingen laserpunktsky publisert for flis { $tile }
progress-point-cloud-cached = bufret punktsky { $tile }
warning-point-cloud-no-cached = ingen bufret punktsky for flis { $tile }; ingen vegetasjonskandidater der
warning-point-cloud-unavailable-cached = punktsky { $tile } ikke tilgjengelig ({ $reason }); bruker den bufrede filen
warning-point-cloud-unavailable-skipped = punktsky { $tile } ikke tilgjengelig ({ $reason }); ingen vegetasjonskandidater der
warning-point-clouds-unavailable = laserpunktskyene er ikke tilgjengelige ({ $reason }); vegetasjonskandidatene bruker de bufrede flisene, hvis det finnes noen
progress-no-point-clouds = ingen laserpunktskyer for vegetasjonen her ennå: delstaten publiserer ingen freemapper kan lese, eller ikke i kartets koordinatsystem
warning-point-clouds-not-fetched = vegetasjonen er valgt, men ingen punktskyer er lastet; kjør `freemapper fetch` for å laste dem
warning-point-cloud-unreadable = punktsky { $tile } kan ikke leses ({ $error }); slett { $path } og kjør `freemapper fetch`
warning-osm-nrw-nearest = ikke noe distriktsutdrag for Nordrhein-Westfalen innen 100 m fra området; leser det nærmeste, { $district }
warning-osm-disk-full-cached = disken er full, så freemapper bruker det bufrede OSM-utdraget fra { $date }. Frigjør plass og kjør `freemapper fetch` på nytt for et nyere
warning-osm-geofabrik-unavailable = Geofabrik ikke tilgjengelig ({ $error }); bruker det bufrede OSM-utdraget fra { $date }
progress-osm-extract-cached = bufret OSM-utdrag { $region } fra { $date }
progress-osm-extract-download = laster ned { $url } (ca. { $megabytes } MB)

## Schleswig-Holstein

progress-sh-bdom-download = laster ned bDOM-flis { $tile } ({ $megabytes } MB, { $index } av { $total })
progress-sh-dgm-not-in-atom = DGM1-flis { $tile } finnes ikke i INSPIRE ATOM-katalogen; laster ned XYZ-filen ({ $megabytes } MB)
progress-sh-dgm-atom-unreadable = DGM1-flis { $tile } fra INSPIRE ATOM-katalogen kan ikke leses ({ $error }); laster ned XYZ-filen ({ $megabytes } MB)
progress-sh-tile = flis { $tile }: { $text }
progress-sh-download-again = flis { $tile }: nedlastingen kan ikke leses ({ $message }); laster den ned én gang til
warning-sh-portal-no-date = Downloadportal gir ingen dato for { $model }-flisen { $tile }
warning-sh-dgm-xyz-used = DGM1-flis { $tile } finnes ikke i INSPIRE ATOM-katalogen; brukte XYZ-filen fra Downloadportal
warning-sh-dgm-xyz-used-unreadable = DGM1-flis { $tile } fra INSPIRE ATOM-katalogen kan ikke leses ({ $error }); brukte XYZ-filen fra Downloadportal

## Tile lists and dates of the height models

warning-model-tile-cells-empty = { $model }-flis { $tile }: { $count } celler i kartområdet har ingen høyde
warning-centre-tile-missing = { $model }-flisen { $tile } i sentrum av kartområdet mangler, så det er ingen dato
warning-centre-tile-unlisted = fliselisten navngir ingen { $model }-flis { $tile } i sentrum av kartområdet, så det er ingen dato
warning-list-unreadable-kept = fliselisten for { $model } kunne ikke leses ({ $error }); brukte den som ble beholdt fra en tidligere kjøring
warning-portal-list-unreadable-kept = Downloadportals fliseliste for { $model } kunne ikke leses ({ $error }); brukte den som ble beholdt fra en tidligere kjøring
warning-surface-much-older = { $model } er fra { $surface }, ortofotoet fra { $flight }. nDSM kan mangle nyere bygninger og trær.

## Rhineland-Palatinate

warning-rp-list-features-unusable = { $count } objekter i fliselisten har ingen brukbar flisnøkkel; hoppet over
warning-rp-list-unreadable-kept = fliselisten fra LVermGeo RP kunne ikke leses ({ $error }); brukte den som ble beholdt fra en tidligere kjøring
warning-rp-metalink-not-kept = Metalink fra LVermGeo RP ble ikke beholdt: { $error }
warning-rp-metalink-unreadable-kept = Metalink fra LVermGeo RP for { $model } kunne ikke leses ({ $detail }); brukte den som ble beholdt fra en tidligere kjøring
warning-rp-metalink-unreadable = Metalink fra LVermGeo RP for { $model } kunne ikke leses ({ $detail }), så flisene som fliselisten ikke gir noe flyår for, blir stående uten høyder
warning-rp-tile-without-name = { $model }-flis { $tile }: denne 1 km-ruten har ingen høyder i kartet. Fliselisten fra LVermGeo RP (Laserbefliegung) { $note } for den, så filnavnet er ikke kjent{ $either ->
        [metalink] , og Metalink navngir heller ingen fil for den
       *[none] {""}
    }
warning-rp-centre-no-date = { $model }-flisen { $tile } har ingen dato: fliselisten fra LVermGeo RP (Laserbefliegung) { $note } for den
word-rp-list-neither = gir verken flyår eller flydager
word-rp-list-no-days = gir ingen flydager
word-rp-list-no-year = gir ikke noe flyår
word-rp-list-days-unreadable = gir flydager { $days } som ikke kan leses
word-rp-list-disagree = gir flydager { $days } og et flyår { $year } som ikke stemmer overens om ett år

## Hesse

warning-hvbg-undated = metadatalisten har ingen lesbar Bildflugdatum for flis { $tiles }; prosjektpostene deres har ingen flydato
warning-hvbg-undated-more = metadatalisten har ingen lesbar Bildflugdatum for flis { $tiles } og { $more } til; prosjektpostene deres har ingen flydato
warning-hvbg-list-unreadable-older = den nye HVBG-metadatalisten { $name } kunne ikke leses ({ $detail }); bruker den eldre listen { $older }

## Lower Saxony

warning-lgln-no-date-unlisted = { $model }: ingen dato, STAC lister ingen flis i sentrum av kartområdet
warning-lgln-no-date = { $model }: ingen dato ({ $error })

## Mecklenburg-Vorpommern

warning-mv-list-failed-date-kept = { $error }; den lagrede modellen beholder sin registrerte dato
warning-mv-cells-empty = { $model }: { $count } celler innenfor flisene i Mecklenburg-Vorpommern har ingen høyde
warning-mv-centre-no-date = { $model }: ingen dato for sentrumsflisen { $tile }

## North Rhine-Westphalia

warning-nrw-list-rows-unusable = { $count } rader i fliselisten for { $model } har ikke noe brukbart flisnavn; hoppet over
warning-nrw-list-rows-undated = { $count } rader i fliselisten for { $model } har ingen lesbar dato
warning-nrw-list-older = tjenerens fliseliste for { $model } er fra { $server }, eldre enn den beholdte fra { $kept }; brukte den beholdte
warning-nrw-list-no-date = fliselisten gir ingen dato for { $model }-flisen { $tile }

## After `init`

note-scale-outside-norm = merk: 1:5000 hører ikke til i sprintnormen; freemapper-print advarer om det, og symbolene blir stående på 100 %

## The detail of an error with many forms: a place, a layer

error-place-not-an-osm-id = { $id } er ikke en OSM-ID som way/992423119
error-place-invalid-answer = ugyldig svar fra stedssøket: { $detail }
error-place-not-found = { $id } ble ikke funnet. Nominatim lister bare OSM-objekter med tagger, så en yttergrense uten tagger i en multipolygon finnes ikke der. Bruk i stedet multipolygon-relasjonen (--osm relation/<id>), eller lagre omrisset som GeoJSON og bruk --geojson
error-place-is-a-point = { $id } er et punkt, ikke et område; velg arealet rundt (en way eller relation)
error-place-no-such-part = { $id } har { $count ->
        [one] 1 del
       *[other] { $count } deler
    }; det finnes ingen del { $part }
error-place-several-parts = { $id } er et område av { $count } atskilte deler ({ $sizes }). Et kartområde er ett polygon: velg en del med --part <n> (1 er den største), eller lagre omrisset du vil ha som GeoJSON og bruk --geojson
error-layer-not-a-world-file = { $path }: ikke en verdensfil
error-layer-file = { $path }: { $detail }
error-layer-detail = { $detail }
error-layer-no-preview = ingen forhåndsvisning for { $layer }; lag med forhåndsvisning: { $names }
error-layer-no-ndsm-published = ingen nDSM er publisert for flisene i dette kartområdet, så det finnes ingen ndsm-mal
error-layer-not-made = { $layer } er ikke laget ennå; kjør { $step ->
        [fetch] freemapper fetch eller run (MCP: start_pipeline); bare Berlin
        [export] freemapper export eller run (MCP: start_pipeline)
       *[derive] freemapper derive eller run (MCP: start_pipeline)
    }
warning-ocad-no-background = { $error }; OCAD-kartet får ingen bakgrunn fra den
warning-map-preview-failed = kartet er skrevet, men forhåndsvisningen mangler: { $error }
warning-training-no-tree-cover = treningskartet beholder åpent land i parker helt: trekronedekket (data/tree-cover.tif) mangler; kjør `freemapper derive` på nytt
