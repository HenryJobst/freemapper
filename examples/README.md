# Beispielprojekte

[English](README.en.md)

Für jedes unterstützte Bundesland gibt es ein Beispiel: ein Gebiet, aus dem freemapper besonders viel ableiten kann
(Altstadt, Schloss oder Park mit Relief, dichte Bebauung, Treppen, Mauern und Hecken). Alle Beispiele sind
Sprintkarten im Maßstab 1:4000 (ISSprOM 2019), erzeugt mit `freemapper run --pdf --ocad`.

| Land | Gebiet | Kartenobjekte | OSM-Kandidaten | Symbole | Vorschau |
|---|---|---:|---:|---:|---|
| Brandenburg | [Park Sanssouci, Potsdam](brandenburg-sanssouci/): Weinbergterrassen, Parterres und Hecken, Schloss | 432 | 785 | 18 | <a href="brandenburg-sanssouci/brandenburg-sanssouci.pdf"><img src="brandenburg-sanssouci/preview.jpg" width="220" alt=""></a> |
| Berlin | [Viktoriapark, Kreuzberg](berlin-viktoriapark/): Kreuzberg mit Wasserfall, Gründerzeitblöcke | 706 | 769 | 19 | <a href="berlin-viktoriapark/berlin-viktoriapark.pdf"><img src="berlin-viktoriapark/preview.jpg" width="220" alt=""></a> |
| Sachsen | [Altstadt Bautzen](sachsen-bautzen/): Altstadt auf dem Felsen über der Spree, steile Talhänge | 1207 | 1180 | 21 | <a href="sachsen-bautzen/sachsen-bautzen.pdf"><img src="sachsen-bautzen/preview.jpg" width="220" alt=""></a> |
| Sachsen-Anhalt | [Schlossberg Quedlinburg](sachsen-anhalt-quedlinburg/): Welterbe-Altstadt, Schlossberg, enge Gassen | 1581 | 953 | 20 | <a href="sachsen-anhalt-quedlinburg/sachsen-anhalt-quedlinburg.pdf"><img src="sachsen-anhalt-quedlinburg/preview.jpg" width="220" alt=""></a> |
| Hamburg | [Treppenviertel Blankenese](hamburg-blankenese/): steiler Elbhang, Treppen und Gassen | 944 | 919 | 18 | <a href="hamburg-blankenese/hamburg-blankenese.pdf"><img src="hamburg-blankenese/preview.jpg" width="220" alt=""></a> |

**Kartenobjekte** sind die sicheren Objekte der OOM-Karte (ALKIS-Gebäude und -Flächen). **OSM-Kandidaten** liegen
als eigene Vorlage bei (`templates/candidates.omap`); im PDF sind sie mitgezeichnet. **Symbole** zählt die
verschiedenen Symbole in beiden zusammen.

## Inhalt eines Beispiels

| Datei | Inhalt |
|---|---|
| `<name>.omap` | das OOM-Startprojekt (OpenOrienteering Mapper) |
| `templates/candidates.omap` | die OSM-Kandidaten als Vorlage |
| `ocad/<name>.ocd`, `ocad/<name>-candidates.ocd` | der OCAD-Ausgangsstand (OCAD 12) |
| `<name>.pdf` | die fertig erzeugte Trainingskarte |
| `preview.jpg` | eine Vorschau der Trainingskarte |
| `freemapper.toml` | das Projekt: Gebiet, Maßstab und die abgerufenen Datenstände |
| `ATTRIBUTION.txt` | die Namensnennung der Datenquellen |

Die Raster-Vorlagen (Luftbild, Schummerung, Hangneigung, nDOM) und die Rohdaten (`data/`) sind nicht im Repository,
weil sie pro Beispiel über 20 MB groß sind. So holen Sie sie zurück:

```sh
freemapper run --ocad --pdf examples/sachsen-anhalt-quedlinburg
```

freemapper lädt die Daten erneut, berechnet die Vorlagen und behält die vorhandenen Karten (`.omap`, `.ocd`)
unverändert. Danach öffnet sich die Karte in OOM und OCAD mit allen Vorlagen.

## Wie die Gebiete ausgewählt wurden

Für jedes Land wurden fünf bis sechs Kandidaten als Sprintkarte erzeugt, darunter Parks, Schulen, Altstädte und
Schlossanlagen. Gezählt wurden die verschiedenen Symbole und die Objekte; die besten zwei bis drei Gebiete jedes
Landes wurden dann an der Trainingskarte verglichen. Gewonnen haben die Gebiete, die das meiste zeigen:
abwechslungsreiche Bebauung, Relief und viele OSM-Kandidaten.

## Lizenzen

Die Karten enthalten abgeleitete amtliche Geodaten und OpenStreetMap-Daten. Die Namensnennung steht in jeder
`ATTRIBUTION.txt`, in den Randangaben des PDF und in den Kartennotizen:

- OpenStreetMap: © OpenStreetMap-Mitwirkende, ODbL;
- Brandenburg: © GeoBasis-DE/LGB, dl-de/by-2-0; Berlin: © Geoportal Berlin, dl-de/zero-2-0;
- Sachsen: GeoSN, dl-de/by-2-0; Sachsen-Anhalt: © GeoBasis-DE / LVermGeo ST, dl-de/by-2-0;
- Hamburg: Freie und Hansestadt Hamburg, LGV, dl-de/by-2-0.

Erstellt mit freemapper – https://github.com/HenryJobst/freemapper
