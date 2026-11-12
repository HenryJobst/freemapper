# Tutorial: ein Adapter in sechs Schritten

Sie bauen einen Ordner-Adapter für eine erfundene Region, „Elbaue“, ein Stück Land um Dessau, und haben am
Ende eine Prüfung, die besteht, und echte Daten von echten Servern. Das dauert etwa zwanzig Minuten. Englisch:
[tutorial.en.md](tutorial.en.md).

Die Region ist erfunden, die Dienste sind es nicht: Es sind die offenen Dienste von Sachsen-Anhalt, damit jede
Anfrage in diesem Tutorial eine Antwort bekommt. Sachsen-Anhalt ist in freemapper schon eingebaut. Die Prüfung
wird Ihnen das sagen, und Sie sollten diesen Adapter nicht in Ihren Adapterordner kopieren: Eine Karte dort
bekäme jedes Objekt doppelt. Für eine eigene Region gehen Sie dieselben Schritte mit den Diensten dieser
Region; die [Anleitung](de.md) sagt, wie Sie sie finden, die [Referenz](manifest-v1.de.md) führt jeden
Schlüssel auf.

Sie brauchen freemapper 0.14.0 oder neuer und einen Texteditor.

## 1. Der Ordner

Ein Adapter ist ein Ordner, der so heißt wie die Kennung des Adapters. Die Kennung ist ein Kleinbuchstabe,
gefolgt von 1 bis 31 Kleinbuchstaben oder Ziffern.

```sh
mkdir -p ~/work/elbaue
cd ~/work
```

## 2. Die Grenze

freemapper fragt einen Adapter nur für Kartengebiete innerhalb seiner Grenze. Die Grenze ist eine
GeoJSON-Datei mit einem Polygon. Für dieses Tutorial genügen sechs Ecken; schreiben Sie sie in
`elbaue/boundary.geojson`:

```json
{
  "type": "Feature",
  "properties": { "name": "Elbaue" },
  "geometry": {
    "type": "Polygon",
    "coordinates": [[
      [12.15, 51.80], [12.33, 51.80], [12.36, 51.86],
      [12.30, 51.90], [12.17, 51.89], [12.15, 51.80]
    ]]
  }
}
```

Die Positionen sind Länge und Breite, und die letzte wiederholt die erste.

## 3. Das Manifest: wer, wo, welche Lizenz, und das Kataster

Schreiben Sie das in `elbaue/manifest.toml`:

```toml
format = 1

[adapter]
id = "elbaue"
revision = 1
min_freemapper = "0.14.0"
crs = 25832
names = { en = "Elbe meadows", de = "Elbaue" }
agency = "LVermGeo"

[net]
hosts = ["www.geodatenportal.sachsen-anhalt.de"]

[boundary]
file = "boundary.geojson"
crs = 4326
simplify_m = 0

[[osm.extract]]
geofabrik = "europe/germany/sachsen-anhalt"
megabytes = 177
label = "Saxony-Anhalt"

[attribution]
licence = "dl-de/by-2-0"
text = "© GeoBasis-DE / LVermGeo ST {year}, dl-de/by-2-0, Daten geändert"
note = "Lizenz dl-de/by-2-0: https://www.govdata.de/dl-de/by-2-0"

[alkis]
kind = "wfs"
url = "https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_ALKIS_WFS_OpenData/guest"
buildings_type = "ave:GebaeudeBauwerk"
land_use_type = "ave:Nutzung"
```

Was die Tabellen sagen:

- `[adapter]`: noch einmal die Kennung, eine Revision, die bei 1 beginnt, das älteste freemapper, das die
  Datei liest, das Koordinatensystem einer Karte in der Region (EPSG:25832, UTM-Zone 32) und die Namen.
- `[net]`: die Server, die der Adapter anfragen darf. freemapper schickt seine Anfragen dorthin und nirgendwo
  sonst hin.
- `[boundary]`: die Datei aus Schritt 2. `crs = 4326` sagt, dass ihre Positionen Grad sind; `simplify_m = 0`
  sagt, dass niemand sie vereinfacht hat.
- `[[osm.extract]]`: der OpenStreetMap-Auszug des Geofabrik-Servers, der die Region abdeckt, mit seiner Größe
  in Megabyte für die Download-Schätzung.
- `[attribution]`: die Lizenz der Daten und die Quellenangabe, die auf jede Karte kommt. Aus `{year}` wird das
  Jahr des Downloads. Das LVermGeo schreibt den Namen `© GeoBasis-DE / LVermGeo ST` vor; die Lizenz verlangt
  ihren eigenen Namen, den Hinweis, dass die Daten verändert wurden, und die Adresse ihres Texts, die `note`
  in `ATTRIBUTION.txt` schreibt.
- `[alkis]`: das Kataster. Der WFS liefert das vereinfachte ALKIS-Schema; die zwei Typnamen stehen in seinen
  Capabilities (`…/guest?SERVICE=WFS&REQUEST=GetCapabilities`).

Jetzt die erste Prüfung. Sie liest den Ordner und fragt keinen Server:

```sh
freemapper adapters check elbaue
```

```
Adapter elbaue (Elbaue), Revision 1, Hosts: www.geodatenportal.sachsen-anhalt.de
Warnung: /home/you/work/elbaue/manifest.toml: adapter.last_verified: the manifest does not say when its services were last checked against it [check.never-verified]
Warnung: /home/you/work/elbaue/manifest.toml: boundary.file: the adapter lies on the ground of Saxony-Anhalt: a map there gets the objects of both [check.shares-ground]
/home/you/work/elbaue besteht die Prüfung, mit 2 Warnungen
```

Der Adapter würde laden. Was am Manifest auffällt, steht auf Englisch da, mit einem festen Code in eckigen
Klammern. Die zweite Warnung ist die oben angekündigte: Die Region liegt in Sachsen-Anhalt. Ein Adapter für
eine Region, die freemapper noch nicht kennt, hat diese Warnung nicht.

## 4. Die Raster

Eine Karte braucht ein Luftbild und ein Geländemodell; ein Oberflächenmodell ergänzt die Vegetationshöhe.
Sachsen-Anhalt liefert alle drei als Standarddienste, also kann das Manifest sie deklarieren. Hängen Sie an:

```toml
[orthophoto]
kind = "wms"
url = "https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_DOP_WMS_OpenData/guest"
layer = "lsa_lvermgeo_dop20_2"
max_px = 4000

[orthophoto.date]
kind = "wfs-geojson"
url = "https://geodatenportal.sachsen-anhalt.de/ows_lvermgeo_wfs_st_geobasis"
layer = "atkis_geobasis_bestand:truedop_kacheln"
field = "BFDATUM"
format = "DD.MM.YYYY"

[terrain]
kind = "wcs"
url = "https://www.geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DGM1"
coverage = "Coverage1"
multipart = true
envelope = [606000, 5646000, 790000, 5882000]
max_m = 4000

[surface]
kind = "wcs"
url = "https://geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DOM1"
coverage = "Coverage1"
multipart = true
envelope = [606000, 5646000, 790000, 5882000]
max_m = 4000
```

- `[orthophoto]`: ein WMS, seine Ebene und die längste Seite, die freemapper in einer Anfrage verlangen darf.
  Ein größeres Gebiet wird in Teilen geholt.
- `[orthophoto.date]`: woher das Flugdatum kommt. Hier hat ein zweiter Dienst eine Ebene mit einer Kachel je
  Bildflugblock und dem Datum im Feld `BFDATUM`.
- `[terrain]` und `[surface]`: je ein WCS. `coverage` und `envelope` stehen in der Antwort auf
  `…/ows_WCS_ST_DGM1?SERVICE=WCS&VERSION=2.0.1&REQUEST=DescribeCoverage&COVERAGEID=Coverage1`: die Kennung
  und die untere und obere Ecke in EPSG:25832. `multipart = true`, weil dieser Server das TIFF in eine
  `multipart/related`-Antwort packt.

Prüfen Sie wieder:

```sh
freemapper adapters check elbaue
```

```
Fehler: /home/you/work/elbaue/manifest.toml:43: orthophoto.date.url: https://geodatenportal.sachsen-anhalt.de/ows_lvermgeo_wfs_st_geobasis: the host geodatenportal.sachsen-anhalt.de is not in `[net] hosts` [policy.host-not-declared]
Fehler: /home/you/work/elbaue/manifest.toml:58: surface.url: https://geodatenportal.sachsen-anhalt.de/ows_WCS_ST_DOM1: the host geodatenportal.sachsen-anhalt.de is not in `[net] hosts` [policy.host-not-declared]
/home/you/work/elbaue besteht die Prüfung nicht
```

Zwei der Dienste liegen auf einem zweiten Host, ohne `www.`. Jeder Befund nennt die Datei, die Zeile, den
Schlüssel und in eckigen Klammern einen Code, den die Referenz erklärt. Ergänzen Sie den Host in `[net]`:

```toml
hosts = ["www.geodatenportal.sachsen-anhalt.de", "geodatenportal.sachsen-anhalt.de"]
```

## 5. Das Luftbild der Gebietsseite, und wer wann geprüft hat

Hängen Sie die Luftbildvorschau an, die die Gebietsseite hinter dem Kartengebiet zeigt, während jemand es
zeichnet:

```toml
[preview]
wms = "https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_DOP_WMS_OpenData/guest"
layer = "lsa_lvermgeo_dop20_2"
format = "image/jpeg"
attribution = "© GeoBasis-DE / LVermGeo ST, dl-de/by-2-0"
agency = "LVermGeo"
```

Und ergänzen Sie in `[adapter]`, unter `agency`, zwei Zeilen mit Ihrem Namen auf GitHub und dem heutigen
Datum:

```toml
maintainers = ["github:your-name"]
last_verified = "2026-11-12"
```

```sh
freemapper adapters check elbaue
```

```
Adapter elbaue (Elbaue), Revision 1, Hosts: www.geodatenportal.sachsen-anhalt.de, geodatenportal.sachsen-anhalt.de
Warnung: /home/you/work/elbaue/manifest.toml: boundary.file: the adapter lies on the ground of Saxony-Anhalt: a map there gets the objects of both [check.shares-ground]
/home/you/work/elbaue besteht die Prüfung, mit einer Warnung
```

## 6. Die Dienste fragen

Bis hierher wurde kein Server gefragt. `--live` holt, was eine Karte eines kleinen Rechtecks bräuchte. Das
Rechteck wird in Grad als `minLon,minLat,maxLon,maxLat` angegeben und liegt innerhalb der Grenze; 150 m mal
150 m genügen. Dieses liegt in der Mitte von Dessau:

```sh
freemapper adapters check elbaue --live --bbox 12.2430,51.8350,12.2452,51.8364
```

```
Adapter elbaue (Elbaue), Revision 1, Hosts: www.geodatenportal.sachsen-anhalt.de, geodatenportal.sachsen-anhalt.de
Warnung: /home/you/work/elbaue/manifest.toml: boundary.file: the adapter lies on the ground of Saxony-Anhalt: a map there gets the objects of both [check.shares-ground]
rasters in Ordnung: Luftbild, Geländemodell, Oberflächenmodell
alkis-buildings in Ordnung: 58 Objekte
alkis-land-use in Ordnung: 60 Objekte
nas nicht angefragt: der Adapter erklärt keine
osm in Ordnung: Saxony-Anhalt, Datenstand 2026-11-11
/home/you/work/elbaue besteht die Prüfung, mit einer Warnung
```

Eine Zeile je Fähigkeit: Die drei Raster sind angekommen, das Kataster hat Gebäude und Nutzung geliefert, und
der OpenStreetMap-Auszug liegt auf dem Server (für die Prüfung wird er nicht heruntergeladen). Die Zahlen und
das Datum sind an Ihrem Tag andere. Der Exit-Code ist 0, wenn der Adapter besteht, 1 bei Befunden und 2, wenn
die Prüfung nicht laufen konnte.

## Was danach kommt

- Um einen Adapter zu benutzen, kopieren Sie seinen Ordner in Ihren Adapterordner; `freemapper adapters list`
  nennt diesen Ordner in seiner letzten Zeile und führt den Adapter dann mit „aus Ihrem Ordner“ auf. Tun Sie
  das nicht mit `elbaue`, aus dem Grund, der oben steht.
- Um einen Adapter allen anzubieten, lesen Sie „Einreichen“ in der [Anleitung](de.md#9-einreichen).
- Eine Region, deren Höhen nicht aus einem WCS kommen, braucht Code; die Anleitung sagt, wann und welchen.
