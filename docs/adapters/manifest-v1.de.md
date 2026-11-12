# Das Adaptermanifest, Format 1

Referenz für `manifest.toml`, die Datei, die einen Regionsadapter beschreibt. Die [Anleitung](de.md) erklärt,
wie man eines schreibt, das [Tutorial](tutorial.de.md) baut eines Schritt für Schritt. Englisch:
[manifest-v1.en.md](manifest-v1.en.md).

Ordner-Adapter liest freemapper ab Version 0.14.0.

## Der Ordner

```
<Kennung>/
  manifest.toml       diese Datei, höchstens 64 KiB
  boundary.geojson    die Grenze, die das Manifest nennt
```

- Der Ordner heißt wie die Kennung des Adapters.
- Er enthält höchstens 64 Dateien. Der Ordner und die Dateien, die freemapper liest, müssen echte sein; einer
  Verknüpfung folgt das Programm nicht.
- Eine `rules.toml` im Ordner wird nicht gelesen (Warnung `adapter.rules-not-read`); der Adapter lädt trotzdem.

## Regeln, die überall gelten

- Ein unbekannter Schlüssel ist ein Befund, ebenso ein Schlüssel, der nicht zur `kind` seiner Tabelle gehört.
- Ein Text hat mindestens ein Zeichen, keinen Zeilenumbruch und kein anderes Steuerzeichen. Die Tabellen unten
  nennen die größte Länge.
- Eine **URL** beginnt mit `https://`, hat keinen Benutzernamen, keinen anderen Port als 443, keine Abfrage
  und kein Fragment (Anfrageparameter gehören in `params`), und ihr Host ist ein DNS-Name aus `[net] hosts`.
  Eine IP-Adresse, `localhost` und ein Name aus nur einem Teil werden abgelehnt. Ein Manifest enthält
  höchstens 64 URLs.
- Ein **Koordinatensystem** ist eine EPSG-Nummer. Eine Karte liegt in 25832 oder 25833, jede Diensttabelle
  auch. Eine Grenzdatei liegt in 4326 oder 25833.
- Eine **Ebene**, ein **Coverage**: 1 bis 100 Buchstaben, Ziffern und `_.:-`. Ein **Objekttyp**: ein Name
  oder `Präfix:Name`. Ein **Feld**: beginnt mit einem Buchstaben oder `_`, dann Buchstaben, Ziffern und `_.-`,
  höchstens 60.
- Die Prüfung macht nach einem Befund weiter. Ein Lauf nennt also alle, jeden mit Datei, Zeile, Schlüssel und
  einem Code.

## `format`

Die erste Zeile der Datei: `format = 1`. Eine andere Zahl ist `manifest.format`.

## `[adapter]`

| Schlüssel | Typ | Pflicht | Bedeutung und Grenzen |
|---|---|---|---|
| `id` | Text | ja | Ein Kleinbuchstabe, dann 1 bis 31 Kleinbuchstaben oder Ziffern. Der Name des Ordners. Die Projektdatei speichert sie. |
| `provider` | Text | nein | Wer die Raster liefert; dasselbe Muster wie `id`, die auch die Vorgabe ist. Der Ordner der Raster im Zwischenspeicher heißt danach. Zwei Adapter teilen sich einen Anbieter nur, wenn beide dasselbe Backend nennen. |
| `revision` | ganze Zahl | ja | Ab 1. Erhöhen Sie sie, wenn Sie das Manifest so ändern, dass sich die Daten ändern. `adapters list` zeigt sie. |
| `min_freemapper` | Text | ja | `x.y.z`: das älteste freemapper, das dieses Manifest liest. Ein älteres Programm lässt den Adapter aus und sagt, welche Version er braucht. Für einen Ordner-Adapter ist das `0.14.0` oder neuer. |
| `crs` | ganze Zahl | ja | 25832 oder 25833: das Koordinatensystem einer Karte, deren Mitte in der Region liegt. |
| `object_id_prefix` | Text | nein | Womit die Objektkennungen des Katasters der Region beginnen (`DEST`, `DESL`): 2 bis 12 Großbuchstaben oder Ziffern. An einer Grenze sagt es, zu welcher Region ein Objekt gehört. |
| `names` | Tabelle | ja | `en` (Pflicht) und `de` (Vorgabe: `en`), je höchstens 80 Zeichen. |
| `agency` | Text | nein | Der Kurzname dessen, der die Daten liefert, für Meldungen; höchstens 40. Vorgabe: `names.en`. |
| `maintainers` | Liste von Texten | nein | Wer sich um den Adapter kümmert; höchstens 8 Einträge mit höchstens 100 Zeichen, zum Beispiel `github:name`. Keine E-Mail-Adresse: Die Datei ist öffentlich. |
| `last_verified` | Text | nein | `JJJJ-MM-TT`: der Tag, an dem die Dienste zuletzt gegen das Manifest geprüft wurden. `adapters check` warnt, wenn er fehlt oder älter als zwölf Monate ist. |
| `licence_of_manifest` | Text | nein | Die Lizenz des Manifests selbst; höchstens 100. |

## `[net]`

| Schlüssel | Typ | Pflicht | Bedeutung und Grenzen |
|---|---|---|---|
| `hosts` | Liste von Texten | ja | Jeder Server, den der Adapter anfragen darf, als DNS-Name, höchstens 32. Kein Platzhalter. Eine Anfrage an einen anderen Host wird abgelehnt, auch eine, zu der eine Umleitung oder ein Link in einer Antwort führt. |

## `[boundary]`

| Schlüssel | Typ | Pflicht | Bedeutung und Grenzen |
|---|---|---|---|
| `file` | Text | ja | Ein Dateiname im Ordner, der auf `.geojson` endet: Buchstaben, Ziffern und `._-`, höchstens 64. Kein Pfad. |
| `crs` | ganze Zahl | ja | 4326 (Länge, Breite) oder 25833. |
| `simplify_m` | Zahl | nein | Wie weit die Grenze vereinfacht wurde, in Metern: 0 oder über 0 bis 1000. Vorgabe 25. |

Die Datei ist GeoJSON: ein `Feature` oder eine `FeatureCollection` mit genau einem Feature, dessen Geometrie
ein `Polygon` oder `MultiPolygon` ist. Höchstens 2 MiB und eine Million Stützpunkte. Die Geometrie muss gültig
sein (kein Ring, der sich selbst kreuzt), und ihre Positionen müssen im genannten Koordinatensystem möglich
sein: Grad bei 4326, Meter bei 25833.

## `[[osm.extract]]`

Mindestens einer, höchstens 16. Der OpenStreetMap-Auszug, den eine Karte in der Region liest.

| Schlüssel | Typ | Pflicht | Bedeutung und Grenzen |
|---|---|---|---|
| `geofabrik` | Text | ja | Der Pfad des Auszugs auf `download.geofabrik.de` ohne `-latest.osm.pbf`, wie in `europe/germany/hessen`. |
| `megabytes` | Zahl | ja | Die Größe des Auszugs, für die Download-Schätzung; über 0. |
| `label` | Text | ja | Wie der Auszug in Meldungen heißt; höchstens 80. |
| `boundary` | Text | bei mehreren Auszügen | Die Grenzdatei dieses Auszugs, benannt wie `[boundary] file`. Ein Adapter mit mehreren Auszügen gibt jedem eine. |
| `cache_prefix` | Text | nein | Womit der Name der Datei im Zwischenspeicher beginnt: Kleinbuchstaben, Ziffern und Bindestriche, höchstens 40. Vorgabe: der letzte Teil von `geofabrik`. Zwei Auszüge können ihn nicht teilen. |

## `[attribution]`

| Schlüssel | Typ | Pflicht | Bedeutung und Grenzen |
|---|---|---|---|
| `licence` | Text | ja | Eine von `dl-de/zero-2-0`, `dl-de/by-2-0`, `cc-by-4.0`, `odbl`, `cc0-1.0`, `other`. |
| `licence_text` | Text | bei `other` | Der Text der Lizenz, höchstens 2000. Nur mit `licence = "other"`. |
| `text` | Text | ja | Die Quellenangabe, höchstens 400. `{year}` darf einmal vorkommen und steht für das Jahr des neuesten Abrufs. Kein anderer Platzhalter. |
| `year` | Text | nein | Nur `newest-fetch`; das geschieht auch ohne den Schlüssel. |
| `short` | Text | nein | Eine kurze Quellenangabe für Stellen, an denen die volle keinen Platz hat; höchstens 60. |
| `note` | Text | nein | Eine Zeile zur Lizenz für `ATTRIBUTION.txt`, zum Beispiel die Adresse des Lizenztexts; höchstens 400. |

## `[alkis]`

Das Kataster: Gebäude und Nutzung im vereinfachten ALKIS-Schema der AdV („ALKIS vereinfacht“). Die Tabelle
kann fehlen, aber ein Adapter hat `[alkis]`, `[orthophoto]` oder beide.

| Schlüssel | Typ | `wfs` | `ogc-api-features` | Bedeutung und Grenzen |
|---|---|---|---|---|
| `kind` | Text | ja | ja | `wfs` oder `ogc-api-features`. |
| `crs` | ganze Zahl | nein | nein | Das Koordinatensystem, in dem der Dienst gefragt wird. Vorgabe: `[adapter] crs`. |
| `name` | Text | nein | – | Der Name des Dienstes in Meldungen; höchstens 80. Vorgabe: `<names.en> ALKIS`. |
| `url` | URL | ja | – | Der WFS, ohne Abfrage. |
| `buildings_type` | Objekttyp | ja | – | Der Typ mit den Gebäuden, wie `ave:GebaeudeBauwerk`. |
| `land_use_type` | Objekttyp | ja | – | Der Typ mit der Nutzung, wie `ave:Nutzung` oder `ave:NutzungFlurstueck`. |
| `square_bez` | Text | nein | – | Die `bez`, die ein Platz ohne eigene bekommt; höchstens 100. Ein solcher Platz bleibt Kandidat. |
| `sorted` | wahr oder falsch | nein | – | `false` für einen WFS, der `SORTBY` mit einer Exception beantwortet; freemapper blättert dann ohne und sortiert die Objekte selbst. Vorgabe `true`. |
| `buildings` | URL | – | ja | Der Items-Endpunkt der Gebäude-Collection. |
| `land_use` | URL | – | ja | Der Items-Endpunkt der Nutzungs-Collection. |
| `normalise` | Text | – | nein | `lgb` (Vorgabe): Der Dienst schreibt die Klassennamen wie die LGB. `hamburg-ascii`: Er schreibt sie in ASCII (`Wohnbauflaeche`). |

`[alkis.fallback]`, nur bei `ogc-api-features`: ein WFS, der gefragt wird, wenn die API ausfällt, im
Koordinatensystem der API. Schlüssel: `url`, `buildings_type`, `land_use_type` (Pflicht), `name`, `square_bez`
und `kind`, die nur `wfs` sein kann.

Der Leser nimmt die Attributnamen des Schemas in Kleinbuchstaben (`oid`, `gebnutzbez`, `funktion`, `nutzart`,
`bez`) oder ganz in Großbuchstaben und die Geometrie aus `geometrie` oder `SHAPE`.

## `[orthophoto]`, `[terrain]`, `[surface]`

Die Raster. Entweder keine der drei Tabellen oder `[orthophoto]` und `[terrain]`, mit oder ohne `[surface]`.
Die Tabellen sind alle Dienste, oder sie nennen alle dasselbe Backend.

Eine Tabelle, die ein Backend nennt, hat einen Schlüssel:

| Schlüssel | Typ | Bedeutung und Grenzen |
|---|---|---|
| `backend` | Text | Der Name eines Rasterbackends, das in freemapper eingebaut ist: 2 bis 32 Kleinbuchstaben, Ziffern und Bindestriche. Ein Name, den das Programm nicht hat, ist `manifest.backend-missing`. `kind = "backend"` darf daneben stehen, kein anderer Schlüssel. |

`[orthophoto]` als Dienst:

| Schlüssel | Typ | `wms` | `wcs` | Bedeutung und Grenzen |
|---|---|---|---|---|
| `kind` | Text | ja | ja | `wms` oder `wcs`. |
| `url` | URL | ja | ja | Der Dienst, ohne Abfrage. |
| `layer` | Ebene | ja | – | Die Ebene des WMS. |
| `coverage` | Coverage | – | ja | Das Coverage des WCS. |
| `max_px` | ganze Zahl | ja | ja | Die längste Seite einer Anfrage in Pixeln, 256 bis 10000. Ein größeres Gebiet wird in Teilen gefragt. |
| `crs` | ganze Zahl | nein | nein | Vorgabe: `[adapter] crs`. |
| `label` | Text | nein | nein | Wie das Raster in der Schätzung und in Meldungen heißt; höchstens 80. Vorgabe: `orthophoto (<names.en> WMS)` oder `… WCS)`. |
| `params` | Tabelle | nein | – | Weitere GetMap-Parameter, siehe unten. |
| `date` | Tabelle | nein | nein | Wo das Flugdatum erfragt wird, siehe unten. |

`[terrain]` und `[surface]` als Dienst sind ein WCS 2.0, der ein GeoTIFF mit Höhen antwortet:

| Schlüssel | Typ | Pflicht | Bedeutung und Grenzen |
|---|---|---|---|
| `kind` | Text | ja | `wcs`. |
| `url` | URL | ja | Der Dienst, ohne Abfrage. |
| `coverage` | Coverage | ja | Die Coverage-Kennung. |
| `envelope` | Liste aus vier Zahlen | ja | Die Ausdehnung des Coverage im Koordinatensystem des Adapters: `[min_e, min_n, max_e, max_n]`. |
| `max_m` | Zahl | ja | Die längste Seite einer Anfrage in Metern, über 99 bis 20000. |
| `multipart` | wahr oder falsch | nein | `true`, wenn das TIFF in einer `multipart/related`-Antwort kommt. Vorgabe `false`. |
| `label` | Text | nein | Vorgabe: `terrain model (<names.en> WCS)` oder `surface model (…)`. |
| `params` | Tabelle | nein | Weitere GetCoverage-Parameter, siehe unten. |
| `date` | Tabelle | nein | Wo das Datum des Modells erfragt wird. |
| `source_label` | Text | nein | Nur `[terrain]`: der Name des Modells im Tag `freemapper:source` der Höhenlinien: 2 bis 40 Kleinbuchstaben, Ziffern und Bindestriche. |

### `params`

Eine Tabelle mit höchstens 16 Anfrageparametern, die freemapper jeder Anfrage der Tabelle anhängt. Ein Name
beginnt mit einem Buchstaben und hat Buchstaben, Ziffern und `_.:-`, höchstens 40; ein Wert hat höchstens 200
Zeichen. Diese setzt freemapper selbst, ein Manifest darf sie also nicht setzen: `service`, `version`,
`request`, `layers`, `styles`, `crs`, `srs`, `bbox`, `width`, `height`, `format`, `coverageid`, `subset`,
`query_layers` (in jeder Schreibweise). Eine TOML-Tabelle kann einen Namen nicht zweimal enthalten. Ein
Dienst, der einen Parameter zweimal braucht, lässt sich deshalb nicht deklarieren.

### `date`

`[orthophoto.date]`, `[terrain.date]`, `[surface.date]`. Ohne die Tabelle hat das Raster kein Datum.

| `kind` | Schlüssel | Was gefragt wird |
|---|---|---|
| `none` | keine | Nichts. |
| `wfs-geojson` | `url`, `layer` (ein Objekttyp), `field`, `format` (`DD.MM.YYYY` oder `YYYY-MM-DD`) | Ein WFS, der GeoJSON antwortet: das `field` des Objekts von `layer` in der Mitte des Gebiets. |
| `stac-newest` | `url` | Eine STAC-API: das neueste Item in der Mitte des Gebiets. |
| `wms-featureinfo` | `layer`, `field` | GetFeatureInfo auf einer Ebene des WMS des Luftbilds. Nur in `[orthophoto.date]` mit `kind = "wms"`. |

## `[[extra.nas]]`

Höchstens einer: ein WFS mit dem vollen ALKIS (NAS), aus dem freemapper Bauwerke, Vegetationsmerkmale und
kleine Gewässer liest.

| Schlüssel | Typ | Pflicht | Bedeutung und Grenzen |
|---|---|---|---|
| `name` | Text | ja | Höchstens 80. |
| `url` | URL | ja | Der WFS. |
| `crs` | ganze Zahl | nein | Vorgabe: `[adapter] crs`. |
| `max_count` | ganze Zahl | ja | Objekte je Anfrage, 1 bis 1 000 000. |
| `max_bytes` | ganze Zahl | ja | Die größte Antwort, die gelesen wird, 1 bis 268 435 456 (256 MiB). |
| `layers` | Liste von Texten | nein | Die Ebenen, die der Dienst liefert, aus `sonstiges-bauwerk`, `vegetationsmerkmal`, `strukturlinie`, `bauwerk-verkehr`, `bauwerk-gewaesser`, `untergeordnetes-gewaesser`. Vorgabe: alle. |
| `namespaces` | Text | nein | Der Parameter `NAMESPACES` der Anfragen; höchstens 300. |

## `[[extra.trees]]`

Höchstens 32: die Baumkataster von Städten, gelesen aus einem GeoJSON-Export von Opendatasoft.

| Schlüssel | Typ | Pflicht | Bedeutung und Grenzen |
|---|---|---|---|
| `id` | Text | ja | 2 bis 32 Kleinbuchstaben, Ziffern und Bindestriche; einmal je Manifest. |
| `name` | Text | ja | Höchstens 80. |
| `crs` | ganze Zahl | nein | Vorgabe: `[adapter] crs`. |
| `envelope` | Liste aus vier Zahlen | ja | Die Ausdehnung der Daten in `crs`. |
| `format` | Text | ja | `ods-geojson-export`. |
| `geo_field` | Feld | ja | Das Feld mit der Position. |
| `endpoints` | Liste von URLs | ja | 1 bis 8. |
| `fields` | Tabelle | ja | `kind`, `id`, `date` (Pflicht), `species`, `crown`: die Namen der Felder. |
| `skipped_kinds` | Liste von Texten | nein | Arten, die keine Bäume sind; höchstens 16. |
| `attribution` | Text | ja | Die Quellenangabe des Katasters, höchstens 400, mit `{year}` höchstens einmal. |
| `max_bytes` | ganze Zahl | ja | 1 bis 268 435 456. |

## `[preview]`

Das Luftbild auf der Karte der Gebietsseite. Die Tabelle kann fehlen.

| Schlüssel | Typ | Pflicht | Bedeutung und Grenzen |
|---|---|---|---|
| `wms` | URL | ja | Ein WMS, der EPSG:3857 liefert. |
| `layer` | Ebene | ja | |
| `format` | Text | ja | `image/png` oder `image/jpeg`. |
| `params` | Tabelle | nein | Wie oben. |
| `attribution` | Text | ja | Die Quellenangabe auf der Karte, höchstens 200, mit `{year}` (dem laufenden Jahr) höchstens einmal. |
| `agency` | Text | ja | Der Kurzname des Amts, für den Namen der Ebene; höchstens 40. |

## `[estimate]`

Megabyte je km² für die Download-Schätzung, wo der Adapter es besser weiß als die Vorgaben (8 für ein Luftbild
aus einem WMS, 4 für ein Höhenmodell aus einem WCS). Jeder Wert über 0 bis 10 000: `orthophoto_mb_per_km2`,
`terrain_mb_per_km2`, `surface_mb_per_km2`.

## Befunde

`freemapper adapters check` und der Lader nennen jeden Befund mit seinem Code. Ein Befund hält den Adapter
draußen, eine Warnung nicht. Die Texte der Befunde sind englisch.

| Code | Was er bedeutet |
|---|---|
| `manifest.too-large` | Das Manifest ist größer als 64 KiB. |
| `manifest.syntax` | Kein TOML. |
| `manifest.format` | `format` fehlt oder ist nicht 1. |
| `manifest.unknown-key` | Ein Schlüssel, den das Format nicht hat, oder einer einer anderen `kind`. |
| `manifest.missing-key` | Ein Schlüssel, den die Tabelle oder ihre `kind` braucht. |
| `manifest.type` | Ein Wert vom falschen Typ. |
| `manifest.id` | Eine Kennung, die nicht zum Muster passt oder nicht der Name des Ordners ist. |
| `manifest.min-freemapper` | Keine Version `x.y.z`. |
| `manifest.crs` | Ein Koordinatensystem, das der Schlüssel nicht erlaubt. |
| `manifest.names` | Ein leerer oder zu langer Name. |
| `manifest.value` | Ein Wert außerhalb seines Bereichs oder Musters. |
| `manifest.file-name` | Ein Dateiname, der kein einfacher Name im Ordner ist. |
| `manifest.geofabrik` | Kein Pfad eines Auszugs auf dem Geofabrik-Server. |
| `manifest.osm` | Die Auszüge passen nicht zusammen. |
| `manifest.licence` | Eine Lizenz außerhalb der Liste oder `other` ohne ihren Text. |
| `manifest.attribution` | Eine Quellenangabe, die leer oder zu lang ist oder einen Zeilenumbruch oder einen zweiten Platzhalter hat. |
| `manifest.kind` | Eine unbekannte `kind`, oder `kind` und `backend` widersprechen sich. |
| `manifest.date-kind` | Eine unbekannte `kind` einer Datumsabfrage, oder `wms-featureinfo` ohne WMS. |
| `manifest.normalise` | Eine unbekannte Voreinstellung für `normalise`. |
| `manifest.params` | Ein Anfrageparameter, den das Manifest nicht setzen darf. |
| `manifest.too-many-urls` | Mehr als 64 URLs. |
| `manifest.empty` | Weder `[alkis]` noch `[orthophoto]`. |
| `manifest.rasters` | Rastertabellen, die nicht zusammenpassen. |
| `manifest.backend-missing` | Ein Rasterbackend, das dieses Programm nicht hat. |
| `manifest.file-missing` | Eine Datei, die das Manifest nennt, fehlt oder lässt sich nicht lesen. |
| `adapter.folder` | Der Ordner ist eine Verknüpfung, hat zu viele Dateien oder lässt sich nicht lesen. |
| `adapter.needs-newer` | Das Manifest verlangt ein neueres freemapper. |
| `adapter.duplicate` | Die Kennung ist vergeben. Ein Ordner kann keinen anderen Adapter ersetzen. |
| `adapter.provider-shared` | Eigene Dienste unter dem Anbieter eines anderen Adapters. |
| `boundary.too-large` | Eine Grenzdatei über 2 MiB oder mit mehr als einer Million Stützpunkten. |
| `boundary.format` | Kein GeoJSON mit einem Polygon-Feature. |
| `boundary.crs` | Positionen, die nicht zum genannten Koordinatensystem gehören können. |
| `boundary.invalid` | Keine gültige Geometrie. |
| `policy.url` | Eine URL, die nicht `https` auf Port 443 mit DNS-Name und ohne Abfrage ist. |
| `policy.host` | Ein Eintrag von `[net] hosts`, der kein DNS-Name ist. |
| `policy.host-not-declared` | Eine URL, deren Host nicht in `[net] hosts` steht. |

Warnungen:

| Code | Was er bedeutet |
|---|---|
| `adapter.rules-not-read` | Der Ordner hat eine `rules.toml`, die dieses freemapper nicht liest. |
| `check.never-verified` | Das Manifest hat kein `last_verified`. |
| `check.verified-long-ago` | `last_verified` liegt mehr als zwölf Monate zurück. |
| `check.built-in-id` | Ein Adapter mit dieser Kennung ist ins Programm eingebaut; aus einem Adapterordner lädt dieser nicht. |
| `check.prefix-shared` | Ein eingebauter Adapter hat dasselbe `object_id_prefix`. |
| `check.boundary-area` | Die Grenze umschließt weniger als 1 km² oder mehr als 2 000 000 km²: wahrscheinlich die falsche Einheit. |
| `check.shares-ground` | Der Adapter liegt auf dem Gebiet eines anderen. Eine Karte dort bekommt die Objekte beider. |

## Vollständige Beispiele

Die Manifeste der eingebauten Bundesländer liegen in
[`crates/adapters/de/manifests/`](https://github.com/HenryJobst/freemapper/tree/main/crates/adapters/de/manifests). Sachsen-Anhalt deklariert jede Quelle
als Dienst. Das Saarland deklariert das Kataster und nennt ein Backend für die Raster. Hamburg nutzt eine OGC
API mit einem WFS als Ersatz. Brandenburg hat das volle ALKIS (`[[extra.nas]]`) und ein Baumkataster. Berlin hat
kein `[alkis]`: Sein Kataster ist Code.
