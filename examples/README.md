# Beispielprojekte

[English](README.en.md)

Für jedes unterstützte Bundesland gibt es ein Beispiel: ein Gebiet, aus dem freemapper besonders viel ableiten kann
(Altstadt, Schloss oder Park mit Relief, dichte Bebauung, Treppen, Mauern und Hecken). Alle Beispiele sind
Sprintkarten im Maßstab 1:4000 (ISSprOM 2019), erzeugt mit `freemapper run --pdf --ocad`.

| Bundesland | Gebiet | Kartenobjekte | Kandidaten | Symbole | Vorschau |
|---|---|---:|---:|---:|---|
| Brandenburg | [Park Sanssouci, Potsdam](brandenburg-sanssouci/): Weinbergterrassen, Parterres und Hecken, Schloss | 469 | 1854 | 27 | <a href="brandenburg-sanssouci/brandenburg-sanssouci.pdf"><img src="brandenburg-sanssouci/preview.jpg" width="220" alt=""></a> |
| Berlin | [Viktoriapark, Kreuzberg](berlin-viktoriapark/): Kreuzberg mit Wasserfall, Gründerzeitblöcke | 789 | 2093 | 34 | <a href="berlin-viktoriapark/berlin-viktoriapark.pdf"><img src="berlin-viktoriapark/preview.jpg" width="220" alt=""></a> |
| Sachsen | [Altstadt Bautzen](sachsen-bautzen/): Altstadt auf dem Felsen über der Spree, steile Talhänge | 1230 | 1289 | 29 | <a href="sachsen-bautzen/sachsen-bautzen.pdf"><img src="sachsen-bautzen/preview.jpg" width="220" alt=""></a> |
| Sachsen-Anhalt | [Schlossberg Quedlinburg](sachsen-anhalt-quedlinburg/): Welterbe-Altstadt, Schlossberg, enge Gassen | 1644 | 1048 | 28 | <a href="sachsen-anhalt-quedlinburg/sachsen-anhalt-quedlinburg.pdf"><img src="sachsen-anhalt-quedlinburg/preview.jpg" width="220" alt=""></a> |
| Hamburg | [Treppenviertel Blankenese](hamburg-blankenese/): steiler Elbhang, Treppen und Gassen | 968 | 947 | 24 | <a href="hamburg-blankenese/hamburg-blankenese.pdf"><img src="hamburg-blankenese/preview.jpg" width="220" alt=""></a> |
| Hessen | [Altstadt und Landgrafenschloss Marburg](hessen-marburg/): steile Oberstadt mit Treppengassen, Schlossberg | 1543 | 945 | 28 | <a href="hessen-marburg/hessen-marburg.pdf"><img src="hessen-marburg/preview.jpg" width="220" alt=""></a> |
| Schleswig-Holstein | [Altstadt Lübeck mit Holstentor](schleswig-holstein-luebeck/): mittelalterliche Altstadtinsel mit Gängen und Höfen | 2294 | 1528 | 28 | <a href="schleswig-holstein-luebeck/schleswig-holstein-luebeck.pdf"><img src="schleswig-holstein-luebeck/preview.jpg" width="220" alt=""></a> |
| Mecklenburg-Vorpommern | [Altstadt Wismar](mecklenburg-vorpommern-wismar/): Welterbe-Altstadt mit Georgenkirche und Marktplatz | 2489 | 1674 | 25 | <a href="mecklenburg-vorpommern-wismar/mecklenburg-vorpommern-wismar.pdf"><img src="mecklenburg-vorpommern-wismar/preview.jpg" width="220" alt=""></a> |
| Nordrhein-Westfalen | [Altstadt Monschau](nordrhein-westfalen-monschau/): Fachwerk-Altstadt im tief eingeschnittenen Rurtal | 999 | 819 | 28 | <a href="nordrhein-westfalen-monschau/nordrhein-westfalen-monschau.pdf"><img src="nordrhein-westfalen-monschau/preview.jpg" width="220" alt=""></a> |
| Niedersachsen | [Großer Garten Herrenhausen, Hannover](niedersachsen-herrenhausen/): Barockgarten mit Hecken, Parterres und Brunnenbecken, daneben der Georgengarten | 285 | 1987 | 28 | <a href="niedersachsen-herrenhausen/niedersachsen-herrenhausen.pdf"><img src="niedersachsen-herrenhausen/preview.jpg" width="220" alt=""></a> |
| Rheinland-Pfalz | [Reichsburg und Altstadt Cochem](rheinland-pfalz-cochem/): Burgberg mit der Reichsburg, Altstadt am steilen Moselufer, gegenüber Cond | 1315 | 790 | 28 | <a href="rheinland-pfalz-cochem/rheinland-pfalz-cochem.pdf"><img src="rheinland-pfalz-cochem/preview.jpg" width="220" alt=""></a> |

**Kartenobjekte** sind die sicheren Objekte der OOM-Karte (ALKIS-Gebäude, -Flächen und -Gewässer, Überdachungen und
Carports). **Kandidaten** liegen als eigene Vorlage bei (`templates/candidates.omap`); im PDF sind sie
mitgezeichnet. Sie kommen aus OSM, aus ALKIS und aus dem Geländemodell, in Berlin zusätzlich aus dem Geoportal Berlin,
in Potsdam aus den Bäumen der Stadtkarte. **Symbole** zählt die verschiedenen Symbole in beiden zusammen.

Flüsse, Kanäle, Teiche und Hafenbecken aus ALKIS stehen als unpassierbares Gewässer (301.1) mit Uferlinie (301.6) in
der Karte. Teiche, Brunnen, Quellen, Bäche und Brücken aus OSM sind Kandidaten. Im Wasser gibt es keine Höhenlinien
und keine Brunnenpunkte, denn die wären meist Fontänen in einem Becken. Wo ALKIS eine Brücke als
Straßenfläche führt, endet das Gewässer an der Brücke mit einer Uferlinie quer zum Fluss; diese Linie entfernen Sie
in OOM.

Überdachungen und Carports aus ALKIS stehen in jedem Bundesland als 522 in der Karte, Durchfahrten und Arkaden in Gebäuden
als 522-Kandidaten. Lübeck hat 93 Überdachungen und 185 Durchfahrten und Arkaden, Wismar 113 Überdachungen und
Carports. In Sachsen und Sachsen-Anhalt führt ALKIS weder Überdachungen noch Durchfahrten, die beiden Beispiele haben
deshalb keine.

In Brandenburg lädt freemapper zusätzlich das vollständige ALKIS (NAS) der LGB. In Sanssouci ergibt das 270 Mauerlinien
(515), 107 Zäune (518), 133 Hecken (410.1), 125 Böschungsoberkanten (104) und 4 Brücken (501.5), alle als Kandidaten.
ALKIS führt oft beide Kanten einer Mauer. Liegen sie näher als 0,55 mm auf dem Papier und einander zugewandt, macht
freemapper daraus eine Linie in der Mitte; in Sanssouci sind das 14 Mittellinien. Dickere Mauern und Kanten, die in
dieselbe Richtung schauen, bleiben doppelt; die zweite Linie löschen Sie in OOM. Hat OSM dieselbe Mauer, Hecke oder
denselben Zaun, bleibt nur die ALKIS-Linie und nennt die OSM-Linie in `freemapper:also`. Läuft eine Linie mit anderem
Symbol parallel und zu nah daneben, etwa eine OSM-Stützmauer auf einer Terrassenmauer oder ein Zaun neben einer Mauer,
verliert die schwächere Linie dieses Stück. Mauerstücke am Gebäudeumriss und Bäume bis 1 m neben einer Hecke, Mauer
oder einem Zaun fallen weg. So sinken in Sanssouci die parallelen Linienpaare unter 1 m Abstand von 162 auf 4. Die
Bäume der Stadtkarte Potsdam sind ein Baumkataster wie in Berlin. Von ihnen bleiben 774 als Punkt, 676 liegen in
Baumgruppenflächen, und OSM-Bäume näher als 2 m an einem Katasterbaum fallen weg.

In Berlin lädt freemapper weitere Ebenen aus dem Geoportal Berlin. Treppen, Mauern, Zäune, Böschungskanten, Denkmäler
und Durchfahrten sind Kandidaten, ebenso die Bäume aus dem Baumkataster, im Viktoriapark 1926 Stück. Dazu kommt die
Karte von Berlin 1:5000 (K5) als Vorlage `templates/k5.png`.

Dazu kommen die Straßenbefahrung (aufgenommen 2014 bis 2016), die Register für Beleuchtung, Zierbrunnen und
Trinkwasserbrunnen und die Gebäudehöhen aus dem Umweltatlas. Die Karte selbst ändert sich dadurch kaum. Im
Viktoriapark schneidet freemapper nur fünf kleine Grünstreifen am Bordstein aus einer ALKIS-Straßenfläche aus. Bei den
Kandidaten kommen 81 Bordsteine (501.1) und 73 vermessene befestigte Flächen (501.5) hinzu. Das sind fast alles Gehwege
und die befestigten Wege im Park, die das PDF jetzt zeigt. Neu sind außerdem Schutzgitter (518), Mauern (515),
Wartehallen (522), Litfaßsäulen (531) sowie Straßenbrunnen und Zierbrunnen (311). Liegt ein Schutzgitter der
Befahrung auf einem OSM-Zaun, bleibt nur das Gitter und nennt den Zaun in `freemapper:also`. Von den 602 Gebäuden
tragen 599 die Grundrissaktualität aus den Gebäudehöhen von 2022 als `freemapper:currency`.

Kleingärten sind in allen Bundesländern Kandidaten für verbotenes Gebiet (520). In Sanssouci betrifft das eine Anlage am
Nordrand, in Wismar und Herrenhausen je eine weitere Fläche.

Punktsymbole dürfen sich nicht berühren. Zwischen zwei Punkten bleiben mindestens 0,15 mm frei, sonst bleibt der
größere und nennt den anderen im Tag `freemapper:generalized`. Stehen drei oder mehr Bäume näher als 2,5 mm (10 m)
beieinander oder berühren sich ihre Kronen fast, macht freemapper daraus eine Fläche: Wald (405) ab 50 %
Kronendeckung, sonst offenes Gelände mit einzelnen Bäumen (402), auf Pflaster ab 9 Bäumen 501.3. Wege und Pflaster
bleiben frei. Baumreihen an Straßen und Alleen bleiben Punkte. In Sanssouci werden so 38 Flächen 402, denn die
Potsdamer Bäume kennen keine Krone, und 5 m Krone decken die Gruppen nicht zur Hälfte. Im Viktoriapark mit seinen
gemessenen Kronen werden es 41 Wälder 405 und 3 Flächen 402. Von den 1926 Katasterbäumen bleiben dort 1128 als
Punkt, 634 liegen in Baumgruppenflächen.

Parks sind gelbes offenes Gelände (401). Parks, Grünanlagen und Grünland aus ALKIS sind Kartenobjekte, in Sanssouci
27 Flächen, im Viktoriapark 5. Wald aus ALKIS wird 405, in Monschau 153 Flächen. Unter allem liegt die belaufbare
Fläche, eine braune Fläche über das ganze Blatt (Symbol 990, Kartenteil `base`). Sie füllt alles, was sonst nichts
zeigt. Punkte in einer Kandidatenfläche 520 zeichnet das PDF nicht, in der Vorlage tragen sie
`freemapper:hidden=520`.

Im Viktoriapark führen die Treppen am Sockel des Nationaldenkmals auf das Denkmal zu. Breite Treppen wie die
Tribünen am Stadion erscheinen als parallele Linien 532.9 mit dem Tag `freemapper:size=wide`. Zäune und Mauern, die
ALKIS und OSM beide haben, stehen nur einmal in den Kandidaten: als ALKIS-Linie, die die OSM-Linie im Tag
`freemapper:also` nennt.

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

Die Raster-Vorlagen (Luftbild, Schummerung, Hangneigung, nDOM, in Berlin die K5) und die Rohdaten (`data/`) sind
nicht im Repository, weil sie pro Beispiel über 20 MB groß sind. So holen Sie sie zurück:

```sh
freemapper run --ocad --pdf examples/sachsen-anhalt-quedlinburg
```

freemapper lädt die Daten erneut, berechnet die Vorlagen und behält die vorhandenen Karten (`.omap`, `.ocd`)
unverändert. Danach öffnet sich die Karte in OOM und OCAD mit allen Vorlagen.

## Wie die Gebiete ausgewählt wurden

Für jedes Bundesland wurden fünf bis sechs Kandidaten als Sprintkarte erzeugt, darunter Parks, Schulen, Altstädte und
Schlossanlagen. Gezählt wurden die verschiedenen Symbole und die Objekte; die besten zwei bis drei Gebiete jedes
Bundeslandes wurden dann an der Trainingskarte verglichen. Gewonnen haben die Gebiete, die das meiste zeigen:
abwechslungsreiche Bebauung, Relief und viele OSM-Kandidaten.

In Niedersachsen hat der Große Garten gewonnen, obwohl die Karte nur 249 sichere Objekte hat. Seine Hecken,
Parterres, Baumreihen und Fontänen ergeben 2223 Kandidaten, 2080 davon aus OSM. Kein anderes Beispiel hat so viele
OSM-Kandidaten.

In Rheinland-Pfalz liefen acht Rechtecke: Koblenz am Deutschen Eck, Trier, Speyer, Mainz, Bad Kreuznach und Cochem.
Bad Kreuznach und Cochem liefen zweimal, denn das erste Rechteck in Bad Kreuznach verfehlte die Nahe, das in Cochem
die Reichsburg. Speyer und Cochem kamen auf je 24 Symbole. Cochem hat gewonnen, weil nur dieses Gebiet starkes Relief
zeigt: dichte Höhenlinien am Burgberg und an den Talhängen, dazu die Mosel und die enge Altstadt. Speyer ist flach.
Koblenz zeigt Rhein und Mosel am Deutschen Eck, kam aber nur auf 23 Symbole und hat kaum Relief.

## Lizenzen

Die Karten enthalten abgeleitete amtliche Geodaten und OpenStreetMap-Daten. Die Namensnennung steht in jeder
`ATTRIBUTION.txt`, in den Randangaben des PDF und in den Kartennotizen:

- OpenStreetMap: © OpenStreetMap-Mitwirkende, ODbL;
- Brandenburg: © GeoBasis-DE/LGB, dl-de/by-2-0; Potsdam: Landeshauptstadt Potsdam / Bäume der Stadtkarte, dl-de/zero-2-0; Berlin: © Geoportal Berlin, dl-de/zero-2-0;
- Sachsen: GeoSN, dl-de/by-2-0; Sachsen-Anhalt: © GeoBasis-DE / LVermGeo ST, dl-de/by-2-0;
- Hamburg: Freie und Hansestadt Hamburg, LGV, dl-de/by-2-0; Hessen: © HVBG, dl-de/zero-2-0; Schleswig-Holstein: © GeoBasis-DE/LVermGeo SH, CC BY 4.0; Mecklenburg-Vorpommern: ©GeoBasis-DE/MV/CC BY 4.0 (Quelle verändert); Nordrhein-Westfalen: © Geobasis NRW, dl-de/zero-2-0; Niedersachsen: © GeoBasis-DE/LGLN, Daten geändert, CC BY 4.0; Rheinland-Pfalz: ©GeoBasis-DE / LVermGeoRP, dl-de/by-2-0, [Daten bearbeitet].

Erstellt mit freemapper – https://github.com/HenryJobst/freemapper
