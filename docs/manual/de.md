# freemapper – Handbuch

[English version](en.md) · [Glossar](../glossary.md) · [README](../../README.md)

Dieses Handbuch ist für Kartierer und Übungsleiter gedacht, die eine Orientierungslaufkarte erstellen
wollen, ohne sich mit Geodaten oder der Kommandozeile zu beschäftigen. Es beschreibt den aktuellen
Stand mit Brandenburg, Berlin und Sachsen.

## Inhalt

1. [Was freemapper macht](#1-was-freemapper-macht)
2. [Was Sie brauchen](#2-was-sie-brauchen)
3. [Installation](#3-installation)
4. [Weg A: mit einem KI-Assistenten](#4-weg-a-mit-einem-ki-assistenten)
5. [Weg B: ohne Assistent](#5-weg-b-ohne-assistent)
6. [Das Kartengebiet im Browser wählen](#6-das-kartengebiet-im-browser-wählen)
7. [Das Ergebnis](#7-das-ergebnis)
8. [Was die Vorlagen zeigen](#8-was-die-vorlagen-zeigen)
9. [Was im Gelände zu prüfen ist](#9-was-im-gelände-zu-prüfen-ist)
10. [Schnelle Trainingskarte (PDF)](#10-schnelle-trainingskarte-pdf)
11. [OCAD](#11-ocad)
12. [Regionen und Grenzgebiete](#12-regionen-und-grenzgebiete)
13. [Später weiterarbeiten](#13-später-weiterarbeiten)
14. [Meldungen und was zu tun ist](#14-meldungen-und-was-zu-tun-ist)
15. [Namensnennung](#15-namensnennung)
16. [Hilfe und Fehler melden](#16-hilfe-und-fehler-melden)

## 1. Was freemapper macht

freemapper holt die amtlichen Geodaten eines Gebiets und baut daraus den **Ausgangsstand** für eine
Schulhofkarte (1:1000) oder Sprintkarte (1:4000). Zu den Geodaten gehören Luftbild, Geländemodell,
Gebäude und Flächennutzung aus dem Liegenschaftskataster (ALKIS). Dazu kommen Daten aus
OpenStreetMap (OSM).

- **Fertig gezeichnet** wird nur, was verlässlich ist: Gebäude, Gewässer aus dem Kataster, auf
  Sprintkarten auch befestigte Flächen (Straßen, Plätze) und unbefestigte Wege mit bekannter Oberfläche.
- **Als Vorlage** kommt alles, was Sie selbst beurteilen müssen: Luftbild, Höhenschummerung, Hangneigung,
  Höhe über Grund (nDOM), Höhenlinien, Zäune, Hecken, Bänke, Spielgeräte, Sperrgebiete, Teiche, Bäche,
  Brunnen und Brücken.
- Gezeichnet und im Gelände geprüft wird weiterhin von Ihnen, in **OpenOrienteering Mapper (OOM)** oder
  **OCAD**.

Auf Wunsch erzeugt freemapper zusätzlich eine **druckfertige Trainingskarte als PDF** (Kapitel 10).

## 2. Was Sie brauchen

- Einen Rechner mit Windows, macOS oder Linux. Ein GIS oder andere Programme brauchen Sie nicht.
- [OpenOrienteering Mapper](https://www.openorienteering.org/apps/mapper/) (kostenlos) oder OCAD 12
  bzw. neuer.
- Internet für das erste Laden.
- Freien Speicherplatz:
  - Beim ersten Mal lädt freemapper den OSM-Auszug des Landes, für Brandenburg und Berlin etwa 300 MB,
    für Sachsen etwa 270 MB, für Sachsen-Anhalt etwa 180 MB, für Hamburg etwa 55 MB, für Hessen etwa 350 MB, für Schleswig-Holstein etwa 160 MB (ohne Hamburg), für Mecklenburg-Vorpommern etwa 130 MB, für Niedersachsen etwa 510 MB (mit Bremen), für Rheinland-Pfalz etwa 275 MB. In Nordrhein-Westfalen sind es die Auszüge der Regierungsbezirke mit je 140 bis 225 MB, bei einem Gebiet an einer Bezirksgrenze zwei davon. Der Auszug wird für
    alle Projekte wiederverwendet.
  - Dazu kommen je Karte meist 10–100 MB.
  - Vor jedem Laden prüft freemapper den freien Platz im Zwischenspeicher und im Projektordner. Liegen
    beide auf verschiedenen Laufwerken, prüft es jedes für sich. Auf jedem Laufwerk lässt es 1 GB frei.
    Reicht der Platz nicht für alles, lässt freemapper das Oberflächenmodell (die nDOM-Vorlage) aus und,
    wenn schon ein älterer OSM-Auszug da ist, auch dessen Aktualisierung. Für jeden ausgelassenen Teil
    warnt es. Kartendaten, Geländemodell und Luftbild braucht jede Karte. Passen sie nicht, bricht
    freemapper ab, bevor es etwas lädt.
- Ein Gebiet in **Brandenburg, Berlin, Sachsen, Sachsen-Anhalt, Hamburg, Hessen, Schleswig-Holstein, Mecklenburg-Vorpommern, Nordrhein-Westfalen, Niedersachsen oder Rheinland-Pfalz**. Bremen und weitere Länder folgen.
- Optional einen KI-Assistenten, z. B. Claude. Dann müssen Sie keine Befehle tippen.

## 3. Installation

freemapper ist ein einzelnes Programm ohne Installer.

1. Laden Sie auf der [Release-Seite](https://github.com/HenryJobst/freemapper/releases/latest) das Archiv
   für Ihr System herunter:
   - **Windows:** `freemapper-…-windows-x86_64.zip`
   - **Mac mit Apple-Chip** (M1 und neuer): `freemapper-…-macos-arm64.tar.gz`
   - **Mac mit Intel-Prozessor:** `freemapper-…-macos-x86_64.tar.gz`
   - **Linux:** `freemapper-…-linux-x86_64.tar.gz`
2. Entpacken Sie das Archiv mit einem Doppelklick. Legen Sie den Ordner an einen festen Ort,
   z. B. `Dokumente/freemapper`.
3. Beim ersten Start warnt das System, weil das Programm nicht signiert ist:
   - **Windows:** „Der Computer wurde durch Windows geschützt“ → *Weitere Informationen* →
     *Trotzdem ausführen*.
   - **macOS:** „kann nicht geöffnet werden …“ → *Systemeinstellungen › Datenschutz & Sicherheit* →
     *Trotzdem öffnen*.

Merken Sie sich den **vollständigen Pfad** zum Programm, z. B. `C:\Users\Anna\Documents\freemapper\freemapper.exe`
oder `/Users/anna/Documents/freemapper/freemapper`. Der Assistent braucht ihn.

## 4. Weg A: mit einem KI-Assistenten

Der Assistent bedient freemapper für Sie. freemapper ist dafür ein „MCP-Server“, ein Werkzeug, das
KI-Assistenten nutzen können.

### Einmal einrichten

- **Claude Desktop:**
  1. *Einstellungen › Entwickler › Konfiguration bearbeiten* öffnen.
  2. In die Datei `claude_desktop_config.json` eintragen (Pfad anpassen, unter Windows mit doppelten
     Backslashes `\\`):

     ```json
     {
       "mcpServers": {
         "freemapper": { "command": "/Users/anna/Documents/freemapper/freemapper", "args": ["mcp"] }
       }
     }
     ```

  3. Claude Desktop neu starten.
- **Claude Code:** im Terminal `claude mcp add freemapper -- /Pfad/zu/freemapper mcp` eingeben.
- **Andere Assistenten:** Bei jedem MCP-fähigen Assistenten tragen Sie dasselbe ein: das Programm mit
  dem Argument `mcp`.

### Eine Karte bestellen

Schreiben Sie dem Assistenten einfach, was Sie brauchen, zum Beispiel:

> Erstelle mit freemapper eine Schulhofkarte für das Evangelische Kreuzgymnasium in Dresden.
> Speichere das Projekt unter Dokumente/OL-Karten/kreuzgymnasium.

> Mach mir eine Sprintkarte für die Altstadt von Strausberg, mit OCAD-Datei und Trainingskarte als PDF.

Der Assistent geht dann so vor:

1. Er sucht den Ort und zeigt Ihnen die Treffer.
2. Er legt das Projekt an.
3. Er sagt Ihnen, wie viel heruntergeladen wird und ob es auf den Datenträger passt.
4. Er startet das Laden und Berechnen. Das dauert beim ersten Mal einige Minuten, später meist weniger
   als eine Minute.
5. Er zeigt Ihnen Vorschaubilder der Vorlagen.

Passt kein Suchtreffer, zum Beispiel weil das Schulgelände in OSM aus zwei Teilen besteht, bitten Sie
den Assistenten, **die Gebietsauswahl im Browser zu öffnen**. Dort zeichnen Sie das Gebiet selbst
(Kapitel 6).

## 5. Weg B: ohne Assistent

Ganz ohne Kommandozeile geht es nicht, aber zwei Befehle genügen. Öffnen Sie ein Terminal, unter
Windows die PowerShell, im Ordner mit freemapper.

1. Das Gebiet im Browser wählen. Der Projektordner wird dabei angelegt:

   ```sh
   ./freemapper select kreuzgymnasium
   ```

   Unter Windows: `.\freemapper.exe select kreuzgymnasium`.
2. Alles laden, berechnen und die Karte schreiben:

   ```sh
   ./freemapper run kreuzgymnasium
   ```

   Mit `--ocad` entsteht zusätzlich die OCAD-Datei, mit `--pdf` die Trainingskarte.

Kennen Sie den OSM-Eintrag des Geländes, geht es auch ohne Browser:

```sh
./freemapper init kreuzgymnasium --osm way/23838018 --title "Evangelisches Kreuzgymnasium"
./freemapper run kreuzgymnasium
```

Besteht das Gelände in OSM aus mehreren getrennten Flächen (eine Multipolygon-Relation mit mehreren äußeren
Ringen), nennt freemapper die Teile mit ihrer Größe. Mit `--part 1` nehmen Sie die größte, mit `--part 2` die
nächste. Die äußeren Wege einer solchen Relation tragen oft keine Tags. Nominatim kennt sie dann nicht, und
`--osm way/…` findet nichts. Nehmen Sie dann die Relation (`--osm relation/…`) oder speichern Sie den Umriss
als GeoJSON und nutzen Sie `--geojson`.

## 6. Das Kartengebiet im Browser wählen

![Gebietsauswahl im Browser](images/area-selector.jpg)

*Die Gebietsauswahl mit dem Kreuzgymnasium Dresden. Kartendaten © OpenStreetMap-Mitwirkende.*

- **Ort suchen:** Namen eingeben, z. B. „Kreuzgymnasium Dresden“, und einen Treffer mit Fläche wählen.
- **Selbst zeichnen:**
  - *Rechteck:* zwei Ecken anklicken.
  - *Polygon:* Ecken nacheinander anklicken. Zum Schließen die erste Ecke anklicken oder doppelklicken.
  - Esc bricht ab, die Rücktaste nimmt die letzte Ecke zurück.
- **Hintergrund:** Oben rechts wählen Sie OpenStreetMap oder das Luftbild (LGB für Brandenburg und
  Berlin, GeoSN für Sachsen).
- **Rechts** sehen Sie:
  - die Größe mit Puffer;
  - die Datenkacheln, die geladen werden (dünne Linien in der Karte);
  - den geschätzten Download;
  - den freien Speicherplatz und wie viel davon der Download braucht. Was nicht auf den Datenträger
    passt, trägt in der Liste „Was geladen wird“ den Vermerk „passt nicht auf den Datenträger“.
    Darüber stehen die Meldungen aus Kapitel 14 und ob freemapper den Teil auslässt oder abbricht.
- **Kartenart, Maßstab und Titel** einstellen, dann *Speichern* und *Fertig*.

Die Seite läuft nur auf Ihrem Rechner. Ein Gebiet darf mit Puffer höchstens 4 km × 4 km groß sein.

## 7. Das Ergebnis

Im Projektordner liegen danach unter anderem:

| Datei | Was sie ist |
|---|---|
| `<name>.omap` | die Karte für OpenOrienteering Mapper, georeferenziert, mit Gebäuden, Gewässern und Randangaben |
| `templates/` | die Vorlagen: Luftbild, Schummerung, Hangneigung, nDOM und die Kandidaten-Karte, in Berlin auch die K5 (Kapitel 12) |
| `ocad/` | mit `--ocad`: die OCAD-Karte und ihre Vorlagen |
| `<name>.pdf` | mit `--pdf`: die Trainingskarte |
| `ATTRIBUTION.txt` | die Namensnennung der Datenquellen |
| `freemapper.toml` | die Einstellungen und der Datenstand des Projekts |

Öffnen Sie die `.omap`-Datei in OOM. Die Vorlagen sind schon eingebunden; im Menü *Vorlagen* blenden Sie
sie ein und aus. Die **Kandidaten** (`templates/candidates.omap`) liegen als eigene Karte darüber. Dort
stehen Zäune, Hecken, Bänke, Höhenlinien, Sperrgebiete, Gewässer aus OSM und Brücken. Prüfen Sie sie
und übernehmen Sie, was stimmt: abzeichnen, oder die Kandidaten-Karte öffnen und die Objekte kopieren
und einfügen. Beide Karten haben dieselbe Georeferenz.

**Gewässer** zeichnet freemapper auf beiden Kartenarten. Flüsse, Kanäle, Seen, Teiche, Hafenbecken und
Meer aus dem Kataster werden zu unpassierbarem Gewässer: einer blauen Fläche (301.1) mit schwarzer
Uferlinie (301.6). Ein Fluss liegt im Kataster oft in vielen einzelnen Flächen. freemapper löst sie zu einer
Fläche auf, damit keine Linien quer über das Wasser laufen. Wo das Wasser über Kartengebiet und Puffer
hinausreicht, bekommt es an dieser Kante keine Uferlinie; Inseln behalten ihre. Gewässer, die im Mittel
schmaler als 2 m sind, kommen nur in die Kandidaten. Die Höhenlinien enden am Ufer dieses Gewässers, in den
Kandidaten wie auf der Trainingskarte.

Aus OSM kommen weitere Kandidaten:

- Teiche, Becken, Schwimmbecken im Freien und Brunnenbecken als Fläche (301). Liegt eine solche Fläche mindestens zur
  Hälfte im Gewässer aus dem Kataster, lässt freemapper sie weg.
- Bäche und Abzugsgräben (`drain`) als Linie: 305, wenn sie ganzjährig Wasser führen, sonst 306. Gräben
  (`ditch`) sind immer 306. Verrohrte Abschnitte fehlen.
- Brunnen (311) und Quellen (312) als Punkt, auf Schulhofkarten auch Trinkbrunnen (313). Brunnenpunkte, die
  im Wasser stehen, lässt freemapper weg, denn meist sind es die Fontänen in einem Becken.
- Brücken als befestigte Fläche (501.5), nur auf Sprintkarten. Sie liegen über dem Wasser.

Meer, das nicht im Kataster steht, bleibt weiß. freemapper liest die Küstenlinie aus OSM nicht.

Unter der Karte stehen die **Randangaben**: Titel, Maßstab, Äquidistanz, Maßstabsleiste und die
Namensnennung. Lassen Sie die Namensnennung auf der gedruckten Karte stehen (Kapitel 15). Die **Nordlinien**
gehören zur Karte und liegen im Kartenteil „default part“. Unterbrechen Sie sie in OOM, wo sie kleine Objekte
verdecken, etwa Steine, Wegenden oder Mauerecken.

Wie das Ergebnis aussieht, zeigen die Beispielprojekte im Repository, eines je Land:
[`examples/`](../../examples/). Dort liegen OOM-Karte, OCAD-Ausgangsstand und PDF-Trainingskarte.

## 8. Was die Vorlagen zeigen

Die Beispiele zeigen das Kreuzgymnasium Dresden. Luftbild, Gelände- und Oberflächenmodell:
Quelle: GeoSN, dl-de/by-2-0, Daten geändert.

| Vorlage | Bild |
|---|---|
| **Luftbild:** Draufsicht, 20 cm je Bildpunkt. Gut für Wege, Plätze und Bewuchs, aber Bäume verdecken den Boden. | ![Luftbild](images/orthophoto.jpg) |
| **nDOM (Höhe über Grund):** gelb 0,3–1 m (Mauern, Bänke, Autos), orange 1–2,5 m (Hecken, Zäune, Spielgeräte), hellgrün 2,5–8 m (Büsche, Schuppen), dunkelgrün über 8 m (Bäume, Häuser). Zeigt, was steht, auch unter Bäumen. | ![nDOM über dem Luftbild](images/ndsm.jpg) |
| **Schummerung:** das Gelände wie von Nordwesten beleuchtet. Zeigt Böschungen, Mulden und Wälle. | ![Schummerung](images/hillshade.jpg) |
| **Hangneigung:** je dunkler, desto steiler. Hilft bei Böschungen und Treppen. | ![Hangneigung](images/slope.jpg) |

Dazu kommen die **Höhenlinien** in der Kandidaten-Karte: auf Schulhofkarten alle 1 m, auf Sprintkarten
alle 2 m, jede fünfte als Zählkurve. Sie sind aus dem Geländemodell berechnet und geglättet. Für die
Karte sind sie ein Entwurf, kein fertiges Ergebnis.

## 9. Was im Gelände zu prüfen ist

freemapper spart das Zeichnen der Grundlage, nicht den Gang ins Gelände. Prüfen Sie vor allem:

- **Gebäude:**
  - Stimmen Umriss und Durchgänge?
  - Fehlen Neubauten, gibt es abgerissene Gebäude noch?
  - Der Datenstand steht bei jedem Gebäude in den Objekt-Tags.
- **Befestigte Flächen** (Sprint): In Sachsen enthalten Straßenflächen auch Grün- und Baumstreifen am Rand.
  In Berlin schneidet freemapper die vermessenen Grünstreifen aus (Kapitel 12); Baumscheiben und Grün, das
  die Straßenbefahrung nicht kennt, bleiben darin. In beiden Ländern sind Plätze nur Kandidaten.
- **Zäune, Mauern, Hecken:** Sie stammen aus OSM und dem nDOM, in Berlin auch aus ALKIS und der
  Straßenbefahrung. Sind sie passierbar? Gibt es Tore?
- **Sperrgebiete:** Privatgrundstücke, Kleingärten, Bahngelände, auf Schulhofkarten alles außerhalb des
  Schulgeländes.
- **Berlin** (Kapitel 12):
  - Überdachungen und Carports aus ALKIS sind Kartenobjekte (522). Gibt es sie noch? Wo eine Überdachung an
    ein Gebäude stößt, schneidet freemapper sie an der Wand ab. Dabei können schmale Reste bleiben.
  - Durchfahrten und Arkaden sind nur Kandidaten. Ist der Durchgang offen, oder hat er ein Tor?
  - Treppen: Die Laufrichtung kommt aus dem Geländemodell. Steigt eine Treppe darin um weniger als 0,3 m,
    rät freemapper. Liegt sie an einem Gebäude oder Denkmal, führt sie darauf zu, sonst läuft sie über ihre
    kurze Seite. Solche Treppen tragen den Tag `freemapper:direction=guessed`. Prüfen Sie sie zuerst. Kurze
    Treppen unter etwa 2 m Länge wirken im Geländemodell mit 1 m Auflösung oft flach.
  - Breite Treppen wie Tribünen zeichnet freemapper als mehrere parallele Linien 532.9 im Abstand einer
    Symbolbreite, mit dem Tag `freemapper:size=wide`. Behalten Sie einige oder zeichnen Sie eine Fläche.
  - Mauern: Hat ALKIS für eine Mauer die Kanten und die Mittellinie, stehen in den Kandidaten zwei oder drei
    parallele Linien 515. Übernehmen Sie eine und löschen Sie die anderen. ALKIS kennt keine Höhe. Ob eine
    Mauer passierbar ist, entscheiden Sie im Gelände.
  - Bäume: Art und Krone stehen im Tag `freemapper:baum`, z. B. `Tilia cordata, Krone 12 m`. Das
    Baumkataster kann älter sein als der Bestand. Bäume auf Schulgeländen und Privatgrund stehen meist nicht
    darin.
  - Straßenbefahrung: Sie wurde 2014 bis 2016 aufgenommen und ist damit rund zehn Jahre alt. Straßen, die
    seither umgebaut wurden, stehen im alten Zustand darin. Prüfen Sie Bordsteine, Geländer, Mauern,
    Möblierung und Grünstreifen im Gelände. Ist ein ausgeschnittener Grünstreifen inzwischen gepflastert,
    bleibt in der befestigten Fläche ein weißer Fleck. Füllen Sie ihn nach dem Luftbild.
  - Geschätzte Aufnahmedaten: Kandidaten mit `freemapper:currency_guessed=yes` und befestigte Flächen mit
    `freemapper:cut_currency_guessed=yes` haben kein lesbares Aufnahmedatum. freemapper setzt dann
    2014-01-01. Prüfen Sie diese Stellen zuerst.
  - Geländer, Verkehrsschutzgitter und Leitplanken sind Kandidaten 518 (unpassierbar). Wo man sie
    übersteigen kann, nehmen Sie 516.
  - Gebäude: Der Datenstand eines Berliner Gebäudes ist seine Grundrissaktualität aus den Gebäudehöhen
    2022, oft Jahre vor dem Laden. Gebäude, die seit 2022 dazugekommen sind, haben kein Datum.
- **Ufer und Brücken:**
  - Die Katasterfläche eines Gewässers kann die Uferböschung einschließen. Legen Sie die Uferlinie
    nach dem Luftbild an die Wasserkante.
  - Die Uferlinie läuft quer über Brücken. Schneiden Sie sie dort von Hand auf.
  - Oft führt das Kataster eine Straßenbrücke als Straßenfläche. Dann endet das Wasser an der Brücke, und
    eine Uferlinie läuft quer über den Fluss. Löschen Sie diese Linie in OOM und ziehen Sie die Fläche unter
    der Brücke durch.
  - Stoßen zwei Gewässerflächen fast aneinander, etwa die zweier Länder an der Grenze mitten im Rhein,
    schließt freemapper Lücken unter 0,5 m. Eine Uferlinie mitten im Fluss steht also für eine breitere
    Lücke im Kataster. Prüfen Sie im Luftbild, ob dort Land ist.
  - Gewässer aus dem Kataster sind immer unpassierbar (301), denn das Kataster kennt keine Tiefe.
    Flaches Wasser, das man durchqueren kann, wird passierbares Gewässer (302).
  - Gewässer aus OSM sind nur Kandidaten. Gibt es den Teich und den Brunnen noch? Führt der Graben
    Wasser?
- **Möblierung und Spielgeräte:** Sie stammen aus OSM, in Berlin auf Schulhofkarten auch aus der
  Straßenbefahrung und dem Beleuchtungskataster. Sie sind oft unvollständig.
- **Höhenlinien:** Kleine Böschungen und Stufen zeigt eher die Schummerung oder Hangneigung.

## 10. Schnelle Trainingskarte (PDF)

Mit `--pdf` (oder „mit Trainingskarte“ beim Assistenten) schreibt freemapper `<name>.pdf`: eine fertig
gezeichnete Karte in A4 oder A3, druckfertig.

![Trainingskarte als PDF](images/training-map.jpg)

*Die Trainingskarte des Kreuzgymnasiums. Quelle: GeoSN, dl-de/by-2-0, Daten geändert; © OpenStreetMap-Mitwirkende.*

Anders als im Ausgangsstand werden hier **alle Kandidaten zu Kartenobjekten**, auch die unsicheren.
Die Karte ist gut für ein schnelles Training. Prüfen Sie sie trotzdem vorher im Gelände.

Die Farben sind CMYK-Werte nach den IOF-Druck- und Farbdefinitionen Rev. 4 (gültig ab 1.12.2024), dieselben
wie in der OOM- und der OCAD-Datei. Druckt Ihr Drucker Braun oder Grün deutlich anders, vergleichen Sie mit dem
IOF Print Test Sheet und passen Sie die Farben an.

## 11. OCAD

Mit `--ocad` (oder „mit OCAD-Datei“) entsteht der Ordner `ocad/`:

- `<name>.ocd`: die Karte im OCAD-12-Format, mit Georeferenz, Gebäuden und den Vorlagen als
  Hintergrundkarten;
- `<name>-candidates.ocd`: die Kandidaten als eigene Karte;
- die Vorlagenbilder mit Weltdateien.

Die Symbole in der OCAD-Datei sind Platzhalter mit richtiger Nummer und Farbe, aber einfacher Grafik.
Laden Sie in OCAD den offiziellen Symbolsatz:

1. *Symbol › Farben und Symbole laden aus …* wählen.
2. Den Satz wählen: ISSprOM 2019 für Sprint, den IOF-Schulsatz 2019 für Schulhofkarten.

OCAD ersetzt die Platzhalter dann nach Nummer. Die OCAD-Ausgabe ist noch nicht in jeder OCAD-Version
geprüft. Rückmeldungen sind willkommen (Kapitel 16).

## 12. Regionen und Grenzgebiete

| Land | Daten von | Besonderheiten |
|---|---|---|
| Brandenburg | LGB | Luftbild, Gelände und nDOM in 1-km-Kacheln; das nDOM ist ein Laser-nDOM mit 20 cm Auflösung |
| Berlin | LGB (Raster), Geoportal Berlin (ALKIS, Baumkataster, K5, Straßenbefahrung, Beleuchtung, Brunnen, Gebäudehöhen) | das nDOM wird aus dem Luftbild-Oberflächenmodell berechnet: etwas weicher, ohne dünne Objekte wie Laternen, dafür mit parkenden Autos; dazu Überdachungen als Kartenobjekte, Treppen, Mauern, Zäune, Hecken und Bäume als Kandidaten und die K5 als Vorlage; die vermessenen Grünstreifen sind aus den Straßenflächen ausgeschnitten, Bordsteine, Geländer, Möblierung, Laternen und Brunnen kommen als Kandidaten, Gebäude tragen ihre Grundrissaktualität (siehe unten) |
| Sachsen | GeoSN | Luftbild aus dem Kartendienst; das nDOM hat nur 1 m Auflösung, Hecken und schmale Mauern sind oft nur angedeutet; unterirdische Bauwerke sind nicht ausfilterbar |
| Sachsen-Anhalt | LVermGeo | Die Karte liegt in UTM-Zone 32 (EPSG:25832), wie alle Daten des Landes; Luftbild aus dem Kartendienst, Gelände- und Oberflächenmodell mit 1 m; das nDOM ist wie in Sachsen grob |
| Hamburg | LGV | Zone 32 wie Sachsen-Anhalt; Luftbild von 2026, Gelände- und Oberflächenmodell aber von 2022: Neubauten der letzten Jahre fehlen im nDOM |
| Hessen | HVBG | Zone 32; Luftbild im Sommer beflogen (2023 Südwesten, 2024 Norden, 2025 Süden); nDOM = DOM1 − DGM1 mit 1 m aus dem Laserscan (2019–2025 je nach Gegend). In vielen Gegenden wurde der Scan im Winter ohne Laub geflogen, daher können die Kronen von Laubbäumen niedriger erscheinen als im Sommer. Der Scan kann aus einem anderen Jahr stammen als das Luftbild (Frankfurt: Scan 28.02.2023, Luftbild 19.08.2025). Die Höhenmodelle kommen aus dem Downloadcenter, eine Zip-Datei je Gemeinde; freemapper liest nur die nötigen 1-km-Kacheln. Plätze ohne `bez` nur als Kandidaten; Straßenflächen mit Grünstreifen (keine Trennung); Schulflächen heißen „Öffentliche Zwecke“ |
| Schleswig-Holstein | LVermGeo SH | Zone 32; das Land wurde in zwei Hälften beflogen: 2024 (Mai/September, belaubt) und 2025 (März–Juni: März/April unbelaubt, Mai/Juni belaubt); Norderstedt z. B. am 17.03.2025 unbelaubt. Der Kartendienst bietet keine Zeitwahl. Gelände: DGM1 mit 1 m aus dem Laserscan (Stand je nach Kachel 2005–2025; Norderstedt 10.03.2022). Oberflächenmodell: bildbasiertes bDOM mit 0,2 m (nur Stände 2023 und 2024); das nDOM ist bDOM − DGM1. freemapper lädt dafür ganze Kacheln von je 105 MB (etwa 90 s je Kachel, Teilladen geht nicht), rechnet sie auf 1 m herunter und behält nur diese Kopie. In der Hälfte von 2025 ist das bDOM zwei Jahre älter als das Luftbild: Neubauten können im nDOM fehlen (freemapper warnt). Das Downloadportal schickt eine unvollständige Zertifikatskette; freemapper bringt das fehlende D-TRUST-Zertifikat mit und verwendet es nur für diesen einen Server. Plätze ohne `bez` nur als Kandidaten; Straßenflächen teils ohne Trennung des Grünstreifens; Schulflächen heißen „Öffentliche Zwecke“; die ALKIS-Schreibweisen der LGB gelten. An der Hamburger Grenze liest freemapper ALKIS und OSM-Auszug beider Seiten; die Rasterdaten kommen aus dem Land, in dem die Mitte des Kartengebiets liegt, und die Kacheln aus Schleswig-Holstein reichen bis nach Hamburg hinein |
| Mecklenburg-Vorpommern | LAiV M-V | Zone 33 (EPSG:25833) wie Brandenburg. Das Luftbild kommt aus dem WCS-Dienst „DOP“ (der Kartendienst trägt ein Wasserzeichen) als JPEG und ist eine Farbpalette mit 256 Farben: leicht körnig, aber zum Abzeichnen geeignet. Beflogen 2021–2025, von März bis Oktober, meist belaubt (etwa 23 % im März/April, dann unbelaubt); Rostock z. B. am 14.08.2024. Gelände und Oberfläche: DGM1 und DOM1 mit 1 m aus dem WCS; das nDOM ist DOM1 − DGM1. Die beiden Modelle können aus verschiedenen Jahren stammen (Rostock: DGM1 März 2024, DOM1 August 2024). Die Höhenmodelle haben keine NoData-Werte: freemapper behält nur die Zellen der 2-km-Kacheln des Landes. Flächen außerhalb (Brandenburg südlich der Kachelkante, offene See außerhalb der Kacheln) bleiben leer; Meer und Bodden innerhalb der Kacheln haben 0 m. ALKIS kommt aus der OGC-API der LGB (wie in Brandenburg), mit WFS als Ersatz; die ALKIS-Schreibweisen der LGB gelten, Schulflächen heißen „Bildung und Wissenschaft“. An der Grenze zu Brandenburg liest freemapper ALKIS und OSM-Auszug beider Seiten; die Rasterdaten kommen aus dem Land, in dem die Mitte des Kartengebiets liegt |
| Nordrhein-Westfalen | Geobasis NRW (Bezirksregierung Köln) | Zone 32 (EPSG:25832) für das ganze Land; das Land reicht weit nach Westen, aber die Karte nutzt nie Zone 33. Das Luftbild kommt aus dem WMS-Dienst, weil die Kacheln nur als JPEG 2000 vorliegen. Beflogen 2022 und 2024 bis 2026, teils unbelaubt, teils belaubt; das Flugdatum gilt je Kachel. Gelände und Oberfläche: DGM1 und DOM1 mit 1 m aus Laserkacheln von 1 km, beide vom selben Flug; Namen und Daten kommen aus der Kachelliste, nicht aus dem Verzeichnis. Der Laserscan pausiert bis Winter 2026/27, der neueste Stand ist Februar 2025; das nDOM ist DOM1 − DGM1. ALKIS aus dem WFS mit den Schreibweisen der LGB und genauerer Nutzungsangabe, Schulgebäude heißen „Allgemein bildende Schule“. OSM aus den Auszügen der fünf Regierungsbezirke (Arnsberg, Detmold, Düsseldorf, Köln, Münster) statt des 917 MB großen Landesauszugs; ein Gebiet an einer Bezirksgrenze liest beide. An der Grenze zu Hessen liest freemapper ALKIS beider Länder; die Rasterdaten kommen aus dem Land, in dem die Mitte des Kartengebiets liegt |
| Niedersachsen | LGLN | Zone 32 (EPSG:25832) für das ganze Land. Das Luftbild kommt aus dem Kartendienst (WMS), Gelände und Oberfläche (DGM1 und DOM1 mit 1 m) kommen aus dem WCS-Dienst. Der WCS-Dienst liefert die Höhenmodelle in Stücken von 2 km. Ein Gebiet von 4 km braucht damit vier Anfragen je Modell und lädt mit dem Luftbild zusammen einige Minuten. Den Datenstand nennt die STAC-Suche des LGLN: das Flugdatum des neuesten Luftbilds und das Datum des neuesten Höhenmodells in der Mitte des Kartengebiets. Beflogen 2024 bis 2026, zu drei Vierteln im Frühjahr (Februar bis April, unbelaubt), sonst von Mai bis Oktober (belaubt). freemapper meldet mit dem Flugdatum, ob die Bäume belaubt waren; Hannover-Südstadt z. B. wurde am 06.03.2025 unbelaubt beflogen. Das nDOM ist DOM1 − DGM1 aus dem Laserscan, beide Modelle vom selben Flug. Der Scan stammt meist von 2016 oder 2017 (an der Küste, an Elbe und Weser von 2025) und ist damit oft 5 bis 9 Jahre älter als das Luftbild (Hannover-Südstadt: Scan 18.04.2016). Ist das DOM1 mehr als zwei Jahre älter, warnt freemapper mit beiden Daten, denn Neubauten und junge Bäume können im nDOM fehlen. ALKIS aus dem WFS mit den Schreibweisen der LGB; Tiefgaragen erkennt freemapper an ihrer Lage unter der Erdoberfläche; Plätze ohne `bez` nur als Kandidaten; Schulgebäude heißen „Allgemein bildende Schule“, Schulflächen „Bildung und Wissenschaft“. Bremen und Bremerhaven gehören noch nicht dazu. Ein Gebiet mit der Mitte in Bremen lehnt freemapper ab. Reicht ein Gebiet aus Niedersachsen nach Bremen hinein, ist das Luftbild dort weiß und ALKIS liefert dort keine Gebäude; freemapper warnt. Der OSM-Auszug Niedersachsen enthält Bremen. An der Hamburger Grenze liest freemapper ALKIS und OSM-Auszug beider Seiten; die Rasterdaten kommen aus dem Land, in dem die Mitte des Kartengebiets liegt |
| Rheinland-Pfalz | LVermGeo RP | Zone 32 (EPSG:25832) für das ganze Land. Das Luftbild kommt aus dem Kartendienst (WMS), denn die Kacheln gibt es nur als JPEG 2000 mit 105 bis 155 MB je 2-km-Block. Der Dienst endet an der Landesgrenze, dahinter ist das Luftbild weiß. Ein Bildflugdatum nennt der Dienst nicht, nur je Block ein Erstellungsdatum. freemapper nimmt es als Flugdatum, denn alle diese Daten liegen zwischen Mai und September, gehäuft an aufeinanderfolgenden Tagen, und die Bilder zeigen belaubte Bäume. Im Projekteintrag steht deshalb „Aktualität (Erstellung)“ beim Datum, in den Meldungen `date of creation`. Beflogen 2022 bis 2025, alle zwei Jahre, immer im Sommer und damit belaubt; das Görres-Gymnasium in Koblenz z. B. am 25.08.2025. Gelände und Oberfläche: DGM1 und DOM1 mit 1 m aus Laserkacheln von 1 km, beide vom selben Flug. Gescannt wurde 2022 bis 2025, jeweils von Februar bis April ohne Laub. Der Dateiname jeder Kachel enthält das Jahr des Scans. freemapper liest es aus der Befliegungsübersicht `Laserbefliegung` des LVermGeo RP, die je Kachel die Flugtage nennt; der letzte Flugtag ist das Datum im Projekteintrag (Koblenz: 06.04.2024). 14 Kacheln nördlich von Trier haben dort kein Flugjahr (Stand November 2026). Für sie liest freemapper die Metalink-Datei des Landes (etwa 11 MB je Modell), aber nur, wenn das Gebiet eine solche Kachel enthält. freemapper behält die Datei und nutzt sie online 30 Tage lang weiter; danach lädt es sie neu. Offline nutzt es die gespeicherte Datei in jedem Alter. Bleibt eine Kachel ohne Namen, bleibt sie leer, und freemapper sagt, warum. Antwortet der Server für eine Kachel der Übersicht mit 404, meldet freemapper sie als fehlend und fragt erst nach 30 Tagen wieder. Die Größenschätzung rechnet mit 1,5 MB je DGM1- und 3 MB je DOM1-Kachel und zählt die Metalink-Datei mit, wenn sie nötig ist. Das nDOM ist DOM1 − DGM1. Scan und Luftbild liegen höchstens drei Jahre auseinander, in einem Viertel der Kacheln ist der Scan sogar neuer. Weil der Scan ohne Laub geflogen wurde, können die Kronen von Laubbäumen im nDOM niedriger erscheinen als im Luftbild. ALKIS aus dem WFS mit den Schreibweisen der LGB; die Nutzung hat genauere Unterarten, z. B. Wohnbaufläche „Offen“ und „Geschlossen“. Tiefgaragen erkennt freemapper an ihrer Lage unter der Erdoberfläche; Plätze ohne `bez` nur als Kandidaten; hat ein Gebäude mehrere Namen, stehen sie mit `;` verbunden in einem Feld. Schulgebäude heißen „Allgemein bildende Schule“, Schulflächen „Bildung und Wissenschaft“. An der Grenze zu Hessen (Mainz und Wiesbaden) liest freemapper ALKIS und OSM-Auszug beider Länder; Wiesbaden und Mainz-Kastel gehören zu Hessen und fehlen im OSM-Auszug Rheinland-Pfalz. Die Rasterdaten kommen aus dem Land, in dem die Mitte des Kartengebiets liegt. Liegt sie in Mainz, ist das Luftbild jenseits des Rheins weiß, die Höhenmodelle reichen aber hinüber |

**Berlin: Bauwerke, Bäume und die K5.** Berührt ein Kartengebiet Berlin, lädt freemapper sechs weitere
Ebenen des Geoportals Berlin (dl-de/zero-2-0): die ALKIS-Bauwerke als Flächen und als Linien, die
Böschungskanten (nur für Sprintkarten), die Hecken und die Straßen- und Anlagenbäume. Sie liegen in
`data/berlin/`. Daraus entstehen:

- Überdachungen und Carports als Kartenobjekte (522) auf beiden Kartenarten. Liegt eine Überdachung zu 90 %
  oder mehr in einem Gebäude, lässt freemapper sie weg. Eine vorhandene Karte bekommt die Überdachungen erst
  mit `run --force`.
- Kandidaten: Treppen (532.6 bis 532.9) mit der Laufrichtung aus dem Geländemodell, Mauern (515),
  Stützmauern (513.2), Zäune (518), Hecken (410.1), Durchfahrten und Arkaden (522) und Denkmäler (526); auf
  Sprintkarten auch Brücken (501.5) und Böschungsoberkanten (104). Das Symbol hängt allein von der
  ALKIS-Bezeichnung ab.
- Bäume aus dem Baumkataster: 417 ab 10 m Kronendurchmesser, sonst 418. Steht ein OSM-Baum weniger als 2 m
  neben einem Katasterbaum, lässt freemapper den OSM-Baum weg.
- Zäune, Mauern, Stützmauern und Hecken stehen oft in ALKIS und in OSM. Liegt eine OSM-Linie zu mindestens 80 %
  ihrer Länge höchstens 1,5 m neben einer ALKIS-Linie mit demselben Symbol, lässt freemapper die OSM-Linie weg.
  Die ALKIS-Linie nennt sie im Tag `freemapper:also`, z. B. `osm/way/123`.

Das Treppensymbol wählt freemapper nach der Breite der Treppe auf dem Papier. Auf Sprintkarten wird eine Treppe ab
3,5 m Breite 532.9, auf Schulhofkarten schon ab etwa 1,1 m. Fast jede Schulhoftreppe ist also 532.9. Ist eine
Treppe breiter als dreimal 532.9 (auf Sprintkarten 12 m), werden daraus parallele Linien 532.9.

Die `Freitreppe`-Linien der ALKIS-Bauwerke sind Stufenkanten. freemapper fasst parallele Kanten mit höchstens 2 m
Abstand, die sich der Länge nach überlappen, zu einer Treppe zusammen. Doppelte Kanten zählen einmal. Die Treppe
läuft quer zu den Kanten, ist so breit, wie die Kanten lang sind, und so lang wie die ganze Folge. Eine einzelne
Kante ergibt keine Treppe.

Jeder Kandidat nennt seine Herkunft in den Objekt-Tags: `freemapper:source=geoportal-berlin/<Ebene>`,
`freemapper:alkis` mit der ALKIS-Bezeichnung (z. B. `Mauerkante, rechts`), bei Bäumen `freemapper:baum` mit
Art und Krone. Die Ebenen haben kein eigenes Datum. Als Datenstand (`freemapper:currency`) steht deshalb das
Datum des Ladens da. Bordsteine aus ALKIS übernimmt freemapper nicht, die der Straßenbefahrung schon.

**Berlin: Straßenbefahrung, Register und Gebäudedaten.** Dazu lädt freemapper die Straßenbefahrung, das
Beleuchtungskataster, die Register der Zier- und der Trinkwasserbrunnen und die Gebäudehöhen 2022 aus
dem Umweltatlas, alle unter dl-de/zero-2-0. Zusammen sind das im Stadtzentrum etwa 6 MB je km² mehr (am
Viktoriapark 5,9 MB). Die Straßenbefahrung hat den öffentlichen Straßenraum Berlins 2014 bis 2016 vermessen.
Eine neuere gibt es nicht, die Daten sind also rund zehn Jahre alt. Deshalb ersetzt sie ALKIS nie. Bis auf den
Grünausschnitt liefert sie nur Kandidaten. Aus diesen Ebenen entstehen:

- Auf Sprintkarten: freemapper schneidet die vermessenen Grünstreifen aus den befestigten Flächen (501.5)
  aus, wenn die Überlappung mindestens 1 mm auf dem Papier breit ist, höchstens aber 2 m im Gelände.
  Baumscheiben bleiben. Jeder Grünstreifen ist außerdem Kandidat 401. Vermessene Gehwege, Fahrbahnen,
  Radwege, Parkflächen und Plätze außerhalb der gezeichneten ALKIS-Flächen werden Kandidaten 501.5, etwa
  Parkwege und Vorplätze. Auch hier bleiben schmale Stücke ab 2 m Breite. Bordsteine werden Kandidaten
  501.1, Werbesäulen 531.
- Auf beiden Kartenarten: Treppen (532.6 bis 532.9, wie die ALKIS-Treppen; das Feld `trs`, vermutlich die
  Stufenzahl, steht als `berlin:trs` dabei), Mauern (515), Geländer, Verkehrsschutzgitter und Leitplanken
  (518, wie Zäune), Fahrgastunterstände (522) und Brunnen (311) aus der Straßenbefahrung und dem
  Zierbrunnen-Register.
- Auf Schulhofkarten: Bänke (543), Hydranten (321), Poller, Masten und Fahnenmasten (544), Spielgeräte
  (541), Trinkbrunnen (313) und die Laternen des Beleuchtungskatasters (545), nur die in Betrieb. Auf
  Sprintkarten fehlt diese Möblierung: Um die Helene-Haeusler-Schule stehen 2 392 Poller in einem km².
- Gebäude: Jedes ALKIS-Gebäude, das die Gebäudehöhen 2022 kennen, bekommt dessen Grundrissaktualität als
  Datenstand (`freemapper:currency`). Das ist der Stand des Grundrisses in diesen Daten, kein
  Vermessungsdatum. Ein Gebäude ohne Treffer bleibt ohne Datum.

Gibt es ein Objekt mehrfach, gewinnt das Register vor ALKIS, ALKIS vor der Straßenbefahrung und diese vor
OSM. Eine Laterne aus dem Beleuchtungskataster ersetzt OSM-Laternen bis 5 m Abstand und vermessene Masten
bis 2 m. Weiter entfernte vermessene Masten tragen meist Schilder oder Ampeln. Ein Brunnen aus einem Register
ersetzt die OSM- und die vermessenen Brunnen bis 3 m. Auf Schulhofkarten ersetzt ein vermessener Hydrant, eine
Bank, ein Mast, ein Fahnenmast oder ein Spielgerät das OSM-Gegenstück bis 2 m. freemapper lässt eine
vermessene Linie weg, die zu mindestens 80 % ihrer Länge höchstens 1,5 m neben einer ALKIS-Linie mit demselben
Symbol liegt, ebenso eine vermessene Treppe, die sich mindestens zur Hälfte mit einer ALKIS-Treppe deckt. Eine
vermessene Mauer neben einer ALKIS-Stützmauer bleibt, denn die Symbole sind verschieden. Das bleibende Objekt
nennt die weggelassenen in `freemapper:also`, ein vermessenes etwa als
`geoportal-berlin/sb-gelaender/Li_2015-05-02_1`. Liegt ein Brunnen aus einem Register oder der
Straßenbefahrung in einem gezeichneten Gewässer, lässt freemapper ihn weg, wie die OSM-Brunnen.

ALKIS und die Straßenbefahrung zerlegen eine Mauer oder ein Geländer oft in kurze Stücke. Stoßen Stücke mit
demselben Symbol Ende an Ende aneinander, fügt freemapper sie zu einer Linie zusammen. Erst danach lässt es
Linien unter 1 mm auf dem Papier weg (bei 1:4000 sind das 4 m). Die zusammengefügte Linie nennt die übrigen
Stücke in `freemapper:also`. Eine vermessene trägt das älteste Aufnahmedatum ihrer Stücke.

Jeder Berliner Kandidat trägt `berlin:layer`. Bei der Straßenbefahrung und den Registern steht dort der
Objekttyp (z. B. `bd_bordstein`, `beleuchtung`), sonst der Name der Ebene (z. B. `bauwerke-linien`). Die
Codes der Straßenbefahrung (`material`, `hms`, `pot`, `bor`, `gws`, `sre`, `hdt`, `trs`) stehen als
`berlin:<Code>` dabei. Was sie bedeuten, ist nicht veröffentlicht. Ein vermessener Kandidat hat als
Datenstand sein Aufnahmedatum aus der `gis_id`. Ist es nicht lesbar, steht dort 2014-01-01 mit
`freemapper:currency_guessed=yes`. Eine beschnittene befestigte Fläche behält ihre ALKIS-Herkunft und nennt
die Grünstreifen an ihrem Rand: `freemapper:cut=geoportal-berlin/sb-gruenflaeche`, `freemapper:cut_ids` und
`freemapper:cut_currency`. Letzteres ist das älteste echte Aufnahmedatum. Gibt es nur geschätzte, kommt
`freemapper:cut_currency_guessed=yes` dazu. Eine vorhandene Karte bekommt Grünausschnitt und Gebäudedaten
erst mit `run --force`.

Die **K5** (Karte von Berlin 1:5000, farbig, Stand 02.01.2026) liegt als Vorlage `templates/k5.png` mit Weltdatei
bei, mit 0,25 m je Bildpunkt. Ist das Gebiet mit Puffer länger als 1,5 km, wird sie gröber. Sie zeigt Parkwege,
Treppen, Mauern, Zäune, Böschungen und Denkmäler und eignet sich zum Vergleichen. In OOM ist sie eingebunden,
aber ausgeblendet; in OCAD ist sie eine Hintergrundkarte. Reicht der Platz auf dem Datenträger nicht, lässt
freemapper sie weg, wie das Oberflächenmodell. Das gilt auch für die Berliner Ebenen; dann bleiben die
gespeicherten.

Für die Laufrichtung der Treppen braucht der Export das Geländemodell. Enthalten die Berliner Ebenen eine
Treppe, speichert `derive` das Geländemosaik deshalb als `data/terrain.tif` im Projekt (etwa 4 MB je
km²), sonst löscht es die Datei. Die Platzprüfung vor dem Laden rechnet die Datei mit den Berliner Ebenen. So arbeitet `export` ohne Internet und ohne Zwischenspeicher.

**Kleingärten** sind in jedem Land Kandidaten für verbotenes Gebiet (520): die ALKIS-Nutzung „Sport-, Freizeit-
und Erholungsfläche“ mit `bez` „Kleingarten“. Parks, Sportanlagen und Spielplätze derselben Nutzung bleiben
offen.

**Koordinatensystem:** Brandenburg, Berlin, Sachsen und Mecklenburg-Vorpommern liefern in UTM-Zone 33, Sachsen-Anhalt, Hamburg, Hessen, Schleswig-Holstein, Nordrhein-Westfalen, Niedersachsen und Rheinland-Pfalz in Zone 32.
Jede Karte bekommt die Zone des Landes, in dem ihre Mitte liegt; OOM und OCAD zeigen sie in der
Georeferenz an. Verschieben Sie ein Kartengebiet über die Zonengrenze oder in ein anderes Land, lädt freemapper alles
neu.

**An einer Landesgrenze** fragt freemapper das Kataster beider Länder ab und liest beide OSM-Auszüge.
Luftbild und Gelände kommen aus dem Land, in dem die Mitte des Kartengebiets liegt. Reicht das Gebiet
weit über die Grenze, bleiben die Vorlagen jenseits davon leer, soweit Luftbild und Höhenmodelle dieses
Landes dort enden. Manche Dienste reichen weiter: Luftbild und Höhenmodelle des LGLN decken auch die
Hamburger Ecke bei Harburg ab, und die Höhenmodelle des LVermGeo RP reichen bei Mainz über den Rhein. freemapper meldet, wenn ein Gebiet über eine Grenze reicht.

## 13. Später weiterarbeiten

- **Ihre Karte ist sicher:** Ein erneutes `run` (oder „bitte neu laden“) ändert eine vorhandene Karte
  nicht. Es aktualisiert nur die Vorlagen und die Kandidaten. Fehlen in der Karte Gebäude oder in Berlin
  die Überdachungen, weist freemapper darauf hin.
- **Neu schreiben:** `--force` ersetzt die Karte. Die alte bleibt als `.omap.bak` erhalten.
- **Gewässer in einer älteren Karte:** Eine Karte, die vor dieser Version entstanden ist, enthält noch
  keine Gewässer. Ein erneutes `run` trägt sie nicht nach. Erst `run --force` schreibt die Karte neu,
  mit Gewässern, wie bei allen anderen Kartenobjekten. Die alte Karte bleibt als `.omap.bak` erhalten,
  die OCAD-Karte als `.ocd.bak`. Jedes `run` erneuert die Kandidaten und die Trainingskarte (`--pdf`).
  Die Trainingskarte zeigt die Gewässer deshalb auch ohne `--force`, die Kandidaten die Gewässer aus
  OSM und die schmalen aus dem Kataster.
- **Gebiet ändern:** `select` auf den vorhandenen Ordner und dann erneut `run`.
- **Ohne Internet:** `--offline` arbeitet mit den bereits geladenen Daten.
- **Datenstand:** freemapper lädt nur neu, was neu veröffentlicht wurde. Ändert sich eine Kachel beim
  Anbieter, gibt es eine Warnung „changed upstream“.

## 14. Meldungen und was zu tun ist

freemapper meldet auf Englisch. Die wichtigsten Meldungen:

| Meldung (Anfang) | Bedeutung | Was tun |
|---|---|---|
| `the map area lies outside the supported states` | Das Gebiet liegt nicht in Brandenburg, Berlin, Sachsen, Sachsen-Anhalt, Hamburg, Hessen, Schleswig-Holstein, Mecklenburg-Vorpommern, Nordrhein-Westfalen, Niedersachsen oder Rheinland-Pfalz. Bremen gehört noch nicht dazu. | Ein anderes Gebiet wählen; weitere Länder folgen. |
| `map area plus buffer is … more than 4000 m per side` | Das Gebiet ist zu groß. | Kleiner zeichnen oder in mehrere Karten teilen. |
| `no area found for …` | Der Ort wurde nicht gefunden, oder das Gelände besteht in OSM aus mehreren Teilen. | Anders schreiben, das Gebiet im Browser zeichnen, oder mit `--osm <id> --part <n>` einen Teil wählen. |
| `… is an area of … separate parts (…)` | Das OSM-Objekt besteht aus mehreren getrennten Flächen. In Klammern stehen ihre Größen, die größte zuerst. | Mit `--part <n>` einen Teil wählen (1 ist der größte), im MCP mit `part`. Oder den Umriss als GeoJSON speichern und `--geojson` nutzen. |
| `… has … parts; there is no part …` | Die Nummer bei `--part` ist größer als die Zahl der Teile. | Eine kleinere Nummer wählen. |
| `… not found. Nominatim lists only tagged OSM objects …` | Nominatim kennt das OSM-Objekt nicht. Oft ist es ein äußerer Weg einer Multipolygon-Relation ohne eigene Tags. | Die Relation nehmen (`--osm relation/…`), oder den Umriss als GeoJSON speichern und `--geojson` nutzen. |
| `--part picks a part of an OSM area and needs --osm` | `--part` gilt nur zusammen mit `--osm`. | Die OSM-Kennung suchen und `--osm <id> --part <n>` angeben. |
| `MultiPolygon with … parts` | Das gewählte Gelände besteht aus mehreren Teilen. | Im Browser ein zusammenhängendes Gebiet zeichnen. |
| `warning: no ALKIS buildings found in the map area` | Im Gebiet gibt es keine Gebäude, z. B. im Wald. | Nichts, wenn das stimmt. Sonst später erneut laden. |
| `the map area reaches beyond the state border` | Ein Teil des Gebiets liegt im Nachbarland. Luftbild und Höhenmodelle kommen aus dem Land der Gebietsmitte, ALKIS und OSM aus beiden Ländern. | Die Vorlagen bleiben leer, wo Luftbild und Höhenmodelle dieses Landes enden. Den Teil von Hand zeichnen. |
| `warning: part of the map area lies outside the supported states` | Ein Teil des Gebiets liegt außerhalb aller unterstützten Länder: im Ausland, auf dem Meer oder in einem Land, das freemapper noch nicht unterstützt. Das sind Bremen, Thüringen, Bayern, Baden-Württemberg und das Saarland. Zum Meer gehören die Ostsee vor Schleswig-Holstein und Mecklenburg-Vorpommern und das schleswig-holsteinische Wattenmeer, etwa zwischen Föhr und Dagebüll. Die Landesgrenzen dieser beiden Länder lassen dieses Wasser aus. Dort gibt es keine Landesdaten, das Luftbild kann weiß sein, und ALKIS liefert weder Gebäude noch Nutzung. freemapper meldet das beim Laden, nicht schon bei der Größenschätzung. Ein Streifen bis 50 m hinter der Grenze löst die Warnung nicht aus. | Den Teil von Hand zeichnen oder das Gebiet verschieben. |
| `download of … failed` oder `invalid ALKIS data` | Ein Server war nicht erreichbar oder hat Unsinn geschickt. | Später erneut versuchen. Bereits geladene Daten bleiben erhalten. |
| `… refused the request (HTTP 400)` | Ein Dienst lehnt die Anfrage ab. | Erneut versuchen. Bleibt es dabei, einen Fehler melden. |
| `… ignored the byte range` | Der Server (Höhenmodelle von Hamburg oder Hessen) schickt das ganze Archiv statt der Kachel; freemapper bricht ab, statt über 1 GB zu laden. | Später erneut versuchen, sonst einen Fehler melden. |
| `download of … is larger than … MB; stopped` | Ein Server (OSM-Auszug, Downloadportal Schleswig-Holstein) schickt weit mehr, als die Datei groß sein kann; freemapper bricht ab und löscht den Teil-Download. | Später erneut versuchen, sonst einen Fehler melden. |
| `no tile of the … covers the map area` | Hamburg: Das landesweite Archiv enthält keine Kachel für das Gebiet; vermutlich wurde es umbenannt. Hessen: Das Downloadcenter hat sich vermutlich geändert. Schleswig-Holstein und Mecklenburg-Vorpommern: Das Downloadportal hat sich vermutlich geändert. Nordrhein-Westfalen: Die Kachelliste von Geobasis NRW nennt keine Kachel für das Gebiet. Rheinland-Pfalz: Nach dem Doppelpunkt steht für jede Kachel der Grund, z. B. `not on the server (404)`, `not in LVermGeo RP's tile list` oder `no flight year in LVermGeo RP's tile list (Laserbefliegung)`. | Einen Fehler melden. |
| `warning: the … has no tile(s) … inside Hamburg` | Im Archiv fehlen einzelne Kacheln. | Die Vorlagen bleiben dort leer. |
| `warning: the … has no tile(s) … inside Hesse` | Im Downloadcenter-Archiv der Gemeinde fehlen einzelne Kacheln. | Die Vorlagen bleiben dort leer. |
| `warning: the Downloadcenter has no file for "…"` | Für den Ort (z. B. ein gemeindefreies Gebiet) gibt es im Downloadcenter keine Datei. Die Kacheln, die danach nach einem Semikolon genannt werden (falls vorhanden), blieben leer, weil keine andere Gemeinde sie enthält. | Kacheln aus Nachbargemeinden brauchen nichts; die genannten Kacheln bleiben leer. |
| `warning: the metadata list has no readable Bildflugdatum for tile …` | In der HVBG-Metadatenliste fehlt für diese Kacheln das Befliegungsdatum. | Es fehlt nur das Datum im Projekteintrag. |
| `warning: the new HVBG metadata list … could not be read …` | Die neu veröffentlichte Liste ließ sich nicht lesen; die ältere gespeicherte Liste wird verwendet. | Bei Dauer freemapper aktualisieren. |
| `warning: the … has no tile(s) … inside Schleswig-Holstein` | Im Downloadportal fehlen einzelne Kacheln. | Die Vorlagen bleiben dort leer. |
| `the Downloadportal lists at most … tiles per request` | (Schleswig-Holstein, Mecklenburg-Vorpommern) Das Gebiet braucht mehr Kacheln, als das Downloadportal je Anfrage nennt. | Das Gebiet kleiner wählen. |
| `the Downloadportal answered with an error page` | Das Downloadportal (Schleswig-Holstein) liefert statt der Kachel eine Fehlerseite. | Später erneut versuchen; freemapper lädt die Kachel einmal selbst erneut. |
| `warning: DGM1 tile … is not in the INSPIRE ATOM directory` / `… does not read (…); used its XYZ from the Downloadportal` | Das GeoTIFF der Geländekachel fehlt im INSPIRE-ATOM-Verzeichnis oder lässt sich nicht lesen (Fehlerseite, anderes Raster, Serverfehler); freemapper hat dieselbe Kachel als XYZ aus dem Downloadportal genommen. Scheitert auch das, nennt der Fehler beide Gründe. | Nichts; die Höhen sind dieselben. |
| `the Downloadportal's tile list … could not be read` | (Schleswig-Holstein, Mecklenburg-Vorpommern) Die Kachelliste ließ sich nicht lesen; die aus einem früheren Lauf wird verwendet. | Nichts, sofern die Daten dieselben sind. |
| `warning: the surface model (bDOM …) is older than the orthophoto (…)` | Das bDOM ist mindestens ein Jahr älter als das Luftbild (in Schleswig-Holstein ab der 2025er Hälfte). | Neubauten im nDOM ggf. von Hand prüfen. |
| `warning: surface model (LVermGeo SH bDOM) unavailable` | Das bDOM ließ sich nicht laden; ein Fehler dabei ist nur eine Warnung. Mit `…; kept the surface model fetched on …` bleibt das Oberflächenmodell eines früheren Abrufs desselben Gebiets erhalten, und das nDOM kommt weiter daraus (auch beim DOM1 der HVBG). | Mit `no nDSM template` fehlt die nDOM-Vorlage; später erneut versuchen. |
| `downloading bDOM tile … (105 MB, … of …)` | Fortschrittsmeldung: freemapper lädt eine ganze Kachel, das dauert etwa 90 s. | Abwarten. |
| `geodaten.schleswig-holstein.de answered with a redirect (HTTP …) to …` | Das Downloadportal leitet auf eine andere Adresse um. freemapper folgt dem nicht, weil es diesen Server nur gegen die eigenen Zertifikate prüft. | Einen Fehler melden. |
| `… certificate …` (Server `geodaten.schleswig-holstein.de`) | Die Zertifikatsprüfung für das Downloadportal schlug fehl, etwa weil dessen Zertifizierungsstelle gewechselt hat. | freemapper aktualisieren, sonst einen Fehler melden. |
| `warning: the … has no tile(s) … inside Mecklenburg-Vorpommern` | Im Downloadportal der Landesvermessung fehlen einzelne Kacheln (nur innerhalb des Landes). | Die Vorlagen bleiben dort leer. |
| `… cells inside the Mecklenburg-Vorpommern tiles have no height` | Einzelne Zellen innerhalb der Landeskacheln haben im Höhenmodell keinen Wert. | Die Vorlagen bleiben dort leer. |
| `… no date for the centre tile …` | Für die Kachel in der Mitte des Kartengebiets nennt das Downloadportal kein Datum (Grenzkarte, Mitte außerhalb der Landeskacheln). | Nur das Datum im Projekteintrag fehlt. |
| `no TIFF in the WCS answer: …` | Der WCS-Dienst der Höhenmodelle von Mecklenburg-Vorpommern oder Niedersachsen hat statt des Bildes eine Fehlermeldung geschickt; ihr Text steht dahinter. In Niedersachsen kann das eine HTML-Seite „MapServer Message“ sein. freemapper versucht es dreimal, bevor es aufgibt. | Später erneut versuchen, bei Dauer einen Fehler melden. |
| `the WCS answered with an error: …` / `not a JPEG (…): …` | Der WCS-Dienst des Luftbilds von Mecklenburg-Vorpommern hat statt des Bildes eine Fehlermeldung oder etwas anderes als ein JPEG geschickt; der bereinigte Anfang der Antwort steht dahinter. | Später erneut versuchen, bei Dauer einen Fehler melden. |
| `…; the stored model keeps its recorded date` | Offline: Die Kachelliste ließ sich nicht lesen; das gespeicherte Höhenmodell wird verwendet und behält das Datum seines Eintrags. | Einmal online laden. |
| `no … stored for this map area` | Offline (Mecklenburg-Vorpommern, Niedersachsen): Es ist kein Höhenmodell gespeichert, das das ganze Gebiet abdeckt. Die Meldung nennt die Datei, die freemapper gesucht hat. | Einmal online laden. |
| `warning: … unavailable (…); no nDSM template` (DOM1 von LAiV M-V, LGLN und LVermGeo RP) | Das DOM1 ließ sich nicht laden; ein Fehler dabei ist nur eine Warnung. | Die nDOM-Vorlage fehlt; später erneut versuchen. |
| `warning: the … has no tile(s) … inside North Rhine-Westphalia` | Einzelne Kacheln des Gebiets fehlen (nur innerhalb des Landes): Entweder nennt die Kachelliste von Geobasis NRW sie nicht, oder sie nennt sie und der Server hat keine Datei (HTTP 404). Ein 404 wird vermerkt, sodass spätere Läufe nicht erneut fragen, bis die Liste eine neue Ausgabe hat. | Die Vorlagen bleiben dort leer. |
| `… tile …: … cells of the map area have no height` | Einzelne Zellen einer Kachel von Geobasis NRW oder LVermGeo RP haben keinen Höhenwert. | Die Vorlagen bleiben dort leer. |
| `the tile list gives no date for the …` / `the … tile … has no date: LVermGeo RP's tile list (Laserbefliegung) …` / `the tile list names no … tile … at the centre of the map area, so there is no date` / `the … tile … at the centre of the map area is missing, so there is no date` | Für die Kachel in der Mitte des Kartengebiets steht in der Kachelliste von Geobasis NRW oder in der Befliegungsübersicht des LVermGeo RP kein Datum oder keine Kachel, oder diese Kachel fehlt. | Nur das Datum im Projekteintrag fehlt. |
| `… rows of the … tile list have no usable tile name; skipped` / `… have no readable date` | Einzelne Zeilen der Kachelliste von Geobasis NRW ließen sich nicht lesen. | Nichts; die übrigen Zeilen gelten. |
| `the tile list of the … could not be read (…); used the one kept from an earlier run` | Die Kachelliste von Geobasis NRW ließ sich nicht lesen; die aus einem früheren Lauf wird verwendet. | Nichts, sofern die Daten dieselben sind. |
| `the server's tile list of the … dates from …, older than the kept one` | Der Server liefert eine ältere Kachelliste als die gespeicherte (ein Spiegel hinter dem Original); freemapper behält die neuere. | Nichts. |
| `warning: no North Rhine-Westphalia district extract within 100 m of the area; reading the nearest, …` | Das Gebiet liegt mehr als 100 m von allen fünf Regierungsbezirken entfernt, etwa knapp hinter der Landesgrenze. freemapper liest den nächsten Auszug. | Nichts; Objekte jenseits davon kommen nicht aus OSM. |
| `download of … is not a complete OSM extract: …` | Ein OSM-Auszug (etwa der eines Regierungsbezirks) ist unvollständig oder beschädigt. freemapper verwirft ihn. | Erneut versuchen; der Auszug wird neu geladen. |
| `the orthophoto was flown on …, with the trees bare` / `…, with the trees in leaf; their crowns hide what lies below them` | Niedersachsen: Fortschrittsmeldung mit dem Flugdatum des Luftbilds. Von Mai bis Oktober gelten die Bäume als belaubt, sonst als unbelaubt. | Bei belaubtem Luftbild Wege, Zäune und kleine Objekte unter den Kronen im Gelände prüfen. |
| `warning: the surface model (LGLN DOM1) is from …, the orthophoto from …` | Das DOM1 ist mehr als 24 Monate älter als das Luftbild, in Niedersachsen meist 5 bis 9 Jahre. Neubauten und junge Bäume können im nDOM fehlen, abgerissene Gebäude noch darin stehen. | Das nDOM mit dem Luftbild vergleichen und Abweichungen im Gelände prüfen. |
| `warning: orthophoto (LGLN WMS): no flight date, …` / `warning: orthophoto (LGLN WMS): the STAC …; kept the stored orthophoto and its flight date …` | Die STAC-Suche des LGLN nennt kein Luftbild in der Mitte des Kartengebiets (`the STAC lists no image …`), oder sie schlug fehl (`the STAC search failed (…)`, der Grund steht in Klammern). Ein gespeichertes Luftbild bleibt mit seinem Flugdatum erhalten, sonst lädt freemapper das Luftbild ohne Datum. Ohne Flugdatum entfällt der Vergleich mit dem DOM1. | Später erneut laden. Bleibt es dabei, einen Fehler melden. |
| `warning: terrain model (LGLN DGM1): no date …` / `warning: surface model (LGLN DOM1): no date …` | Die STAC-Suche nennt kein Höhenmodell in der Mitte des Kartengebiets, oder sie schlug fehl (der Grund steht in Klammern, z. B. `not a STAC search answer: …`). Das Modell selbst ist geladen. | Nichts; nur das Datum im Projekteintrag fehlt. Ohne Datum des DOM1 entfällt der Vergleich mit dem Luftbild. |
| `the terrain model (LGLN DGM1) has no height in the map area` / `the surface model (LGLN DOM1) has no height …` | Der WCS-Dienst des LGLN lieferte für das ganze Gebiet nur Zellen ohne Höhe. freemapper schreibt keine Datei. Beim DGM1 bricht das Laden ab, beim DOM1 fehlt die nDOM-Vorlage. | Später erneut versuchen. Liegt das Gebiet in Niedersachsen und bleibt es dabei, einen Fehler melden. |
| `warning: the … has no tile(s) … inside the map area` | Einzelne Kacheln des Gebiets fehlen. Entweder nennt die Befliegungsübersicht `Laserbefliegung` sie nicht (gemeldet nur für Kacheln mit der Mitte im Land), oder sie nennt kein Flugjahr und die Metalink-Datei keine Datei, oder der Server antwortet mit 404. freemapper vermerkt einen 404 und fragt erst nach 30 Tagen wieder. | Die Vorlagen bleiben dort leer. |
| `warning: … tile …: this 1 km square has no heights in the map` | Rheinland-Pfalz: Die Befliegungsübersicht nennt für diese Kachel kein Flugjahr oder keine lesbaren, zueinander passenden Flugtage, daher ist der Dateiname unbekannt. Endet die Meldung mit `and its Metalink names no file for it either`, kennt auch die Metalink-Datei die Kachel nicht. | Die Vorlagen bleiben dort leer. Bleibt es dabei, einen Fehler melden. |
| `the LVermGeo RP tile list could not be read (…); used the one kept from an earlier run` | Die Befliegungsübersicht ließ sich nicht laden; freemapper nimmt die aus einem früheren Lauf für dasselbe Gebiet. | Nichts, sofern die Daten dieselben sind. |
| `… features of the tile list have no usable tile key; skipped` | Einzelne Einträge der Befliegungsübersicht ließen sich nicht lesen. | Nichts; die übrigen Einträge gelten. |
| `the LVermGeo RP Metalink of the … could not be read (…); used the one kept from an earlier run` / `…, so the tiles its tile list gives no flight year for stay without heights` | Die Metalink-Datei ließ sich nicht laden. Bei der ersten Meldung nimmt freemapper die gespeicherte, bei der zweiten bleiben die Kacheln ohne Flugjahr leer. | Später erneut laden. |
| `… not a tile list (Laserbefliegung): …` / `… not a GeoTIFF: …` / `… not a Metalink: …` / `the tile list holds … of the … tiles it matched` | Ein Server des LVermGeo RP hat statt der Befliegungsübersicht, der Kachel oder der Metalink-Datei etwas anderes geschickt, meist eine HTML-Fehlerseite; der bereinigte Anfang der Antwort steht dahinter. Die letzte Meldung heißt, dass die Übersicht unvollständig ankam. | Später erneut versuchen, bei Dauer einen Fehler melden. |
| `orthophoto unchanged (date of creation …)` | Rheinland-Pfalz: freemapper verwendet das gespeicherte Luftbild weiter, weil der Kartendienst dasselbe Erstellungsdatum nennt (oder gerade keines und das Bild jünger als 30 Tage ist). Dieses Datum gilt als Flugdatum. | Nichts. |
| `downloading Berlin ALKIS structures, tree cadastre, street survey and point layers` | Fortschrittsmeldung: freemapper lädt die Berliner Ebenen, im Stadtzentrum etwa 10 MB je km² (am Viktoriapark 9,8 MB). | Abwarten. |
| `warning: Berlin … unavailable (…); keeping the stored layer` / `…; layer skipped` | Eine Ebene des Geoportals Berlin (z. B. `bauwerke-linien` oder `sb-moebel`) ließ sich nicht laden. Fasst eine Ebene mehrere Objekttypen zusammen, wie `sb-moebel` die Bänke, Poller und Hydranten, fehlt sie ganz, wenn einer davon scheitert. Mit `keeping the stored layer` bleibt die gespeicherte Ebene, die das Gebiet noch abdeckt. Mit `layer skipped` fehlen ihre Kandidaten. Das Laden geht weiter. | Später erneut laden. |
| ``no Berlin ALKIS structures stored; run `freemapper fetch` online …`` | Offline: Für das Berliner Gebiet sind keine Bauwerke, Bäume und Überdachungen gespeichert. | Einmal online laden. |
| `removing the stored Berlin layers: the map area is no longer near Berlin` | Das Gebiet liegt nicht mehr an Berlin. freemapper löscht die gespeicherten Berliner Ebenen. | Nichts. |
| `warning: the K5 map template is unavailable (…); skipped` | Die K5 ließ sich nicht laden. Die Vorlage fehlt, das Laden geht weiter. | Später erneut laden. |
| `k5 is not made yet; run freemapper fetch or run …` | Assistent oder Vorschau fragen nach der K5-Vorlage, aber es gibt sie nicht: Das Gebiet liegt nicht in Berlin, es wurde noch nicht geladen, oder das Laden schlug fehl. | In Berlin `fetch` oder `run` ausführen. |
| `warning: the map area now lies in another state's data` | Das Gebiet liegt jetzt bei einem anderen Anbieter. | Nichts; freemapper lädt alles neu. |
| `offline and not cached` / `no OSM extract cached` | Mit `--offline` fehlen Daten. | Einmal mit Internet laden. |
| `no tiles fetched yet` / `the map area needs tiles … that were not fetched` | Es wurde noch nicht geladen, oder das Gebiet hat sich geändert. | `run` bzw. `fetch` ausführen. |
| `map already exists (it may contain your work)` | Es gibt schon eine Karte. | `--force` nur benutzen, wenn sie ersetzt werden soll. |
| `The map lacks the Berlin canopies and carports (522) that are now map objects` | Die Karte stammt aus der Zeit vor den Berliner Überdachungen. freemapper hat sie behalten, die Überdachungen fehlen darin. | Mit `run --force` neu schreiben. Die alte Karte bleibt als `.omap.bak`. Oder die Überdachungen aus den Kandidaten von Hand übernehmen. |
| `warning: … changed upstream` | Der Anbieter hat neue Daten veröffentlicht. | Nichts; die Vorlagen sind jetzt aktuell. |
| `warning: freemapper skips the … for lack of disk space` | Für diesen Teil (Oberflächenmodell, Aktualisierung des OSM-Auszugs, die Berliner Ebenen mit Bauwerken, Bäumen, Straßenbefahrung und Registern oder die Berliner K5) ist auf dem Datenträger kein Platz, wenn 1 GB frei bleiben soll. freemapper lässt ihn aus. Ein gespeichertes Oberflächenmodell desselben Gebiets bleibt, ebenso der ältere OSM-Auszug und gespeicherte Berliner Ebenen. Die Meldung nennt, wie viel der Teil braucht und wie viel frei ist. | Platz schaffen und `freemapper fetch` erneut ausführen. |
| `freemapper keeps the surface model fetched on … for the nDSM template` / `freemapper keeps the cached surface model of tile(s) … for the nDSM template` | Folgt auf die Meldung davor: Das Oberflächenmodell eines früheren Abrufs (bzw. die genannten Kacheln aus dem Zwischenspeicher) deckt das Gebiet noch ab, die nDOM-Vorlage kommt weiter daraus. | Nichts. |
| `not enough disk space for the …` | Für Kartendaten, Geländemodell oder Luftbild ist kein Platz. Ohne sie entsteht keine Karte, deshalb bricht freemapper ab, bevor es etwas lädt. | Platz schaffen oder den Zwischenspeicher mit `--cache-dir` auf ein anderes Laufwerk legen, dann erneut starten. |
| `the disk is full, so freemapper could not write …` | Der Datenträger lief beim Schreiben voll, etwa weil ein anderes Programm gleichzeitig Platz belegt hat. freemapper löscht die halbe Datei, im Zwischenspeicher bleibt nichts Kaputtes. | Platz schaffen und erneut versuchen. |
| `warning: the disk is full, so freemapper uses the cached OSM extract from …` | Der neue OSM-Auszug passte nicht mehr auf den Datenträger. freemapper nimmt den älteren. | Platz schaffen und später erneut laden. |
| `warning: cannot read the free disk space of …` | Das System nennt den freien Platz nicht, etwa bei manchen Netzlaufwerken. freemapper lädt ohne Prüfung. | Nichts. Läuft der Datenträger voll, meldet freemapper `the disk is full`. |

## 15. Namensnennung

Die Geodaten stehen unter offenen Lizenzen, verlangen aber eine Namensnennung. freemapper schreibt sie
in die Randangaben, in die Kartennotizen und in `ATTRIBUTION.txt`, z. B.:

- `© OpenStreetMap contributors, ODbL`
- `© GeoBasis-DE/LGB <Jahr>, dl-de/by-2-0, Daten geändert` (Brandenburg; in Berlin mit dem Zusatz
  `© Geoportal Berlin, dl-de/zero-2-0`)
- `© Geoportal Berlin / ALKIS Berlin, dl-de/zero-2-0, Daten geändert` (Berlin, mit ALKIS-Daten); dazu
  `Geoportal Berlin / Karte 1:5000 von Berlin (K5), dl-de/zero-2-0`, wenn die K5 geladen wurde, und
  `Geoportal Berlin / Baumbestand Berlin, dl-de/zero-2-0`, wenn Bäume aus dem Kataster kamen; außerdem
  `Geoportal Berlin / Straßenbefahrung 2014, dl-de/zero-2-0, Daten geändert`, wenn Objekte der
  Straßenbefahrung geladen wurden, `Geoportal Berlin / Öffentliche Beleuchtung, Zierbrunnen,
  Trinkwasserbrunnen, dl-de/zero-2-0` mit nur den Registern, die Objekte in Karte oder Kandidaten geliefert
  haben (auf einer Sprintkarte etwa `Geoportal Berlin / Zierbrunnen, dl-de/zero-2-0`), und
  `Geoportal Berlin / Gebäudehöhen (Umweltatlas), dl-de/zero-2-0`, wenn Gebäudehöhen geladen wurden
- `Quelle: GeoSN, dl-de/by-2-0, Daten geändert` (Sachsen)
- `© GeoBasis-DE / LVermGeo ST <Jahr>, dl-de/by-2-0, Daten geändert` (Sachsen-Anhalt)
- `Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV) <Jahr>, dl-de/by-2-0, Daten geändert` (Hamburg)
- `Geobasisdaten © Hessische Verwaltung für Bodenmanagement und Geoinformation (HVBG) <Jahr>, dl-de/zero-2-0, verändert` (Hessen)
- `© GeoBasis-DE/LVermGeo SH/CC BY 4.0 (Quelle verändert)` und der Lizenzlink https://creativecommons.org/licenses/by/4.0/ (Schleswig-Holstein)
- `©GeoBasis-DE/MV/CC BY 4.0 (Quelle verändert)` und der Lizenzlink https://creativecommons.org/licenses/by/4.0/ (Mecklenburg-Vorpommern)
- `Geobasisdaten © Geobasis NRW (Bezirksregierung Köln) <Jahr>, dl-de/zero-2-0` (Nordrhein-Westfalen; die Lizenz verlangt weder Namensnennung noch Änderungsvermerk, freemapper nennt die Quelle trotzdem)
- `© GeoBasis-DE/LGLN <Jahr>, Daten geändert, CC BY 4.0` und der Lizenzlink https://creativecommons.org/licenses/by/4.0/ (Niedersachsen; `<Jahr>` ist das Jahr des Downloads)
- `©GeoBasis-DE / LVermGeoRP<Jahr>, dl-de/by-2-0, www.lvermgeo.rlp.de [Daten bearbeitet]` und der Lizenzhinweis `Lizenz dl-de/by-2-0 (Datenlizenz Deutschland – Namensnennung – Version 2.0): https://www.govdata.de/dl-de/by-2-0` (Rheinland-Pfalz; `<Jahr>` ist das Jahr des Downloads, der Lizenzhinweis steht nur in den Kartennotizen und in `ATTRIBUTION.txt`)
- `Erstellt mit freemapper – https://github.com/HenryJobst/freemapper`

Lassen Sie diese Zeilen auf gedruckten und veröffentlichten Karten stehen.

## 16. Hilfe und Fehler melden

Fragen, Fehler und Wünsche bitte als [Issue auf GitHub](https://github.com/HenryJobst/freemapper/issues).
Hilfreich sind:

- die Meldung;
- der Ort (Name oder Koordinaten);
- die Datei `freemapper.toml` des Projekts.
