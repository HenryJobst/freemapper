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

- **Fertig gezeichnet** wird nur, was verlässlich ist: Gebäude, auf Sprintkarten auch befestigte Flächen
  (Straßen, Plätze) und unbefestigte Wege mit bekannter Oberfläche.
- **Als Vorlage** kommt alles, was Sie selbst beurteilen müssen: Luftbild, Höhenschummerung, Hangneigung,
  Höhe über Grund (nDOM), Höhenlinien, Zäune, Hecken, Bänke, Spielgeräte und Sperrgebiete.
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
    für Sachsen etwa 270 MB, für Sachsen-Anhalt etwa 180 MB, für Hamburg etwa 55 MB, für Hessen etwa 350 MB, für Schleswig-Holstein etwa 160 MB (ohne Hamburg). Er wird für
    alle Projekte wiederverwendet.
  - Dazu kommen je Karte meist 10–100 MB.
- Ein Gebiet in **Brandenburg, Berlin, Sachsen, Sachsen-Anhalt, Hamburg, Hessen oder Schleswig-Holstein**. Weitere Länder folgen.
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
3. Er sagt Ihnen, wie viel heruntergeladen wird.
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
  - den geschätzten Download.
- **Kartenart, Maßstab und Titel** einstellen, dann *Speichern* und *Fertig*.

Die Seite läuft nur auf Ihrem Rechner. Ein Gebiet darf mit Puffer höchstens 4 km × 4 km groß sein.

## 7. Das Ergebnis

Im Projektordner liegen danach unter anderem:

| Datei | Was sie ist |
|---|---|
| `<name>.omap` | die Karte für OpenOrienteering Mapper, georeferenziert, mit Gebäuden und Randangaben |
| `templates/` | die Vorlagen: Luftbild, Schummerung, Hangneigung, nDOM und die Kandidaten-Karte |
| `ocad/` | mit `--ocad`: die OCAD-Karte und ihre Vorlagen |
| `<name>.pdf` | mit `--pdf`: die Trainingskarte |
| `ATTRIBUTION.txt` | die Namensnennung der Datenquellen |
| `freemapper.toml` | die Einstellungen und der Datenstand des Projekts |

Öffnen Sie die `.omap`-Datei in OOM. Die Vorlagen sind schon eingebunden; im Menü *Vorlagen* blenden Sie
sie ein und aus. Die **Kandidaten** (`templates/candidates.omap`) liegen als eigene Karte darüber. Dort
stehen Zäune, Hecken, Bänke, Höhenlinien und Sperrgebiete. Prüfen Sie sie und übernehmen Sie, was
stimmt: abzeichnen, oder die Kandidaten-Karte öffnen und die Objekte kopieren und einfügen. Beide
Karten haben dieselbe Georeferenz.

Unter der Karte stehen die **Randangaben**: Titel, Maßstab, Äquidistanz, Maßstabsleiste, Nordlinien
und die Namensnennung. Lassen Sie die Namensnennung auf der gedruckten Karte stehen (Kapitel 15).

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
- **Befestigte Flächen** (Sprint): In Berlin und Sachsen enthalten Straßenflächen auch Grün- und
  Baumstreifen am Rand. Plätze sind dort nur Kandidaten.
- **Zäune, Mauern, Hecken:** Sie stammen aus OSM und dem nDOM. Sind sie passierbar? Gibt es Tore?
- **Sperrgebiete:** Privatgrundstücke, Bahngelände, auf Schulhofkarten alles außerhalb des Schulgeländes.
- **Möblierung und Spielgeräte:** Sie stammen aus OSM und sind oft unvollständig.
- **Höhenlinien:** Kleine Böschungen und Stufen zeigt eher die Schummerung oder Hangneigung.

## 10. Schnelle Trainingskarte (PDF)

Mit `--pdf` (oder „mit Trainingskarte“ beim Assistenten) schreibt freemapper `<name>.pdf`: eine fertig
gezeichnete Karte in A4 oder A3, druckfertig.

![Trainingskarte als PDF](images/training-map.jpg)

*Die Trainingskarte des Kreuzgymnasiums. Quelle: GeoSN, dl-de/by-2-0, Daten geändert; © OpenStreetMap-Mitwirkende.*

Anders als im Ausgangsstand werden hier **alle Kandidaten zu Kartenobjekten**, auch die unsicheren.
Die Karte ist gut für ein schnelles Training. Prüfen Sie sie trotzdem vorher im Gelände.

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
| Berlin | LGB (Raster), Geoportal Berlin (ALKIS) | das nDOM wird aus dem Luftbild-Oberflächenmodell berechnet: etwas weicher, ohne dünne Objekte wie Laternen, dafür mit parkenden Autos |
| Sachsen | GeoSN | Luftbild aus dem Kartendienst; das nDOM hat nur 1 m Auflösung, Hecken und schmale Mauern sind oft nur angedeutet; unterirdische Bauwerke sind nicht ausfilterbar |
| Sachsen-Anhalt | LVermGeo | Die Karte liegt in UTM-Zone 32 (EPSG:25832), wie alle Daten des Landes; Luftbild aus dem Kartendienst, Gelände- und Oberflächenmodell mit 1 m; das nDOM ist wie in Sachsen grob |
| Hamburg | LGV | Zone 32 wie Sachsen-Anhalt; Luftbild von 2026, Gelände- und Oberflächenmodell aber von 2022: Neubauten der letzten Jahre fehlen im nDOM |
| Hessen | HVBG | Zone 32; Luftbild im Sommer beflogen (2023 Südwesten, 2024 Norden, 2025 Süden); nDOM = DOM1 − DGM1 mit 1 m aus dem Laserscan (2019–2025 je nach Gegend). In vielen Gegenden wurde der Scan im Winter ohne Laub geflogen, daher können die Kronen von Laubbäumen niedriger erscheinen als im Sommer. Der Scan kann aus einem anderen Jahr stammen als das Luftbild (Frankfurt: Scan 28.02.2023, Luftbild 19.08.2025). Die Höhenmodelle kommen aus dem Downloadcenter, eine Zip-Datei je Gemeinde; freemapper liest nur die nötigen 1-km-Kacheln. Plätze ohne `bez` nur als Kandidaten; Straßenflächen mit Grünstreifen (keine Trennung); Schulflächen heißen „Öffentliche Zwecke“ |
| Schleswig-Holstein | LVermGeo SH | Zone 32; das Land wurde in zwei Hälften beflogen: 2024 (Mai/September, belaubt) und 2025 (März–Juni: März/April unbelaubt, Mai/Juni belaubt); Norderstedt z. B. am 17.03.2025 unbelaubt. Der Kartendienst bietet keine Zeitwahl. Gelände: DGM1 mit 1 m aus dem Laserscan (Stand je nach Kachel 2005–2025; Norderstedt 10.03.2022). Oberflächenmodell: bildbasiertes bDOM mit 0,2 m (nur Stände 2023 und 2024); das nDOM ist bDOM − DGM1. freemapper lädt dafür ganze Kacheln von je 105 MB (etwa 90 s je Kachel, Teilladen geht nicht), rechnet sie auf 1 m herunter und behält nur diese Kopie. In der Hälfte von 2025 ist das bDOM zwei Jahre älter als das Luftbild: Neubauten können im nDOM fehlen (freemapper warnt). Das Downloadportal schickt eine unvollständige Zertifikatskette; freemapper bringt das fehlende D-TRUST-Zertifikat mit und verwendet es nur für diesen einen Server. Plätze ohne `bez` nur als Kandidaten; Straßenflächen teils ohne Trennung des Grünstreifens; Schulflächen heißen „Öffentliche Zwecke“; die ALKIS-Schreibweisen der LGB gelten. An der Hamburger Grenze liest freemapper ALKIS und OSM-Auszug beider Seiten; die Rasterdaten kommen aus dem Land, in dem die Mitte des Kartengebiets liegt, und die Kacheln aus Schleswig-Holstein reichen bis nach Hamburg hinein |

**Koordinatensystem:** Brandenburg, Berlin und Sachsen liefern in UTM-Zone 33, Sachsen-Anhalt, Hamburg, Hessen und Schleswig-Holstein in Zone 32.
Jede Karte bekommt die Zone des Landes, in dem ihre Mitte liegt; OOM und OCAD zeigen sie in der
Georeferenz an. Verschieben Sie ein Kartengebiet über die Zonengrenze oder in ein anderes Land, lädt freemapper alles
neu.

**An einer Landesgrenze** fragt freemapper das Kataster beider Länder ab und liest beide OSM-Auszüge.
Luftbild und Gelände kommen aus dem Land, in dem die Mitte des Kartengebiets liegt. Reicht das Gebiet
weit über die Grenze, bleiben die Vorlagen jenseits davon leer. freemapper weist darauf hin.

## 13. Später weiterarbeiten

- **Ihre Karte ist sicher:** Ein erneutes `run` (oder „bitte neu laden“) ändert eine vorhandene Karte
  nicht. Es aktualisiert nur die Vorlagen und die Kandidaten. Fehlen in der Karte Gebäude, weist
  freemapper darauf hin.
- **Neu schreiben:** `--force` ersetzt die Karte. Die alte bleibt als `.omap.bak` erhalten.
- **Gebiet ändern:** `select` auf den vorhandenen Ordner und dann erneut `run`.
- **Ohne Internet:** `--offline` arbeitet mit den bereits geladenen Daten.
- **Datenstand:** freemapper lädt nur neu, was neu veröffentlicht wurde. Ändert sich eine Kachel beim
  Anbieter, gibt es eine Warnung „changed upstream“.

## 14. Meldungen und was zu tun ist

freemapper meldet auf Englisch. Die wichtigsten Meldungen:

| Meldung (Anfang) | Bedeutung | Was tun |
|---|---|---|
| `the map area lies outside the supported states` | Das Gebiet liegt nicht in Brandenburg, Berlin, Sachsen, Sachsen-Anhalt, Hamburg, Hessen oder Schleswig-Holstein. | Ein anderes Gebiet wählen; weitere Länder folgen. |
| `map area plus buffer is … more than 4000 m per side` | Das Gebiet ist zu groß. | Kleiner zeichnen oder in mehrere Karten teilen. |
| `no area found for …` | Der Ort wurde nicht gefunden, oder das Gelände besteht in OSM aus mehreren Teilen. | Anders schreiben, oder das Gebiet im Browser zeichnen. |
| `MultiPolygon with … parts` | Das gewählte Gelände besteht aus mehreren Teilen. | Im Browser ein zusammenhängendes Gebiet zeichnen. |
| `warning: no ALKIS buildings found in the map area` | Im Gebiet gibt es keine Gebäude, z. B. im Wald. | Nichts, wenn das stimmt. Sonst später erneut laden. |
| `the map area reaches beyond the state border` | Ein Teil des Gebiets liegt im Nachbarland. | Die Vorlagen bleiben dort leer. |
| `download of … failed` oder `invalid ALKIS data` | Ein Server war nicht erreichbar oder hat Unsinn geschickt. | Später erneut versuchen. Bereits geladene Daten bleiben erhalten. |
| `… refused the request (HTTP 400)` | Ein Dienst lehnt die Anfrage ab. | Erneut versuchen. Bleibt es dabei, einen Fehler melden. |
| `… ignored the byte range` | Der Server (Höhenmodelle von Hamburg oder Hessen) schickt das ganze Archiv statt der Kachel; freemapper bricht ab, statt über 1 GB zu laden. | Später erneut versuchen, sonst einen Fehler melden. |
| `download of … is larger than … MB; stopped` | Ein Server (OSM-Auszug, Downloadportal Schleswig-Holstein) schickt weit mehr, als die Datei groß sein kann; freemapper bricht ab und löscht den Teil-Download. | Später erneut versuchen, sonst einen Fehler melden. |
| `no tile of the … covers the map area` | Hamburg: Das landesweite Archiv enthält keine Kachel für das Gebiet; vermutlich wurde es umbenannt. Hessen: Das Downloadcenter hat sich vermutlich geändert. Schleswig-Holstein: Das Downloadportal hat sich vermutlich geändert. | Einen Fehler melden. |
| `warning: the … has no tile(s) … inside Hamburg` | Im Archiv fehlen einzelne Kacheln. | Die Vorlagen bleiben dort leer. |
| `warning: the … has no tile(s) … inside Hesse` | Im Downloadcenter-Archiv der Gemeinde fehlen einzelne Kacheln. | Die Vorlagen bleiben dort leer. |
| `warning: the Downloadcenter has no file for "…"` | Für den Ort (z. B. ein gemeindefreies Gebiet) gibt es im Downloadcenter keine Datei. Die Kacheln, die danach nach einem Semikolon genannt werden (falls vorhanden), blieben leer, weil keine andere Gemeinde sie enthält. | Kacheln aus Nachbargemeinden brauchen nichts; die genannten Kacheln bleiben leer. |
| `warning: the metadata list has no readable Bildflugdatum for tile …` | In der HVBG-Metadatenliste fehlt für diese Kacheln das Befliegungsdatum. | Es fehlt nur das Datum im Projekteintrag. |
| `warning: the new HVBG metadata list … could not be read …` | Die neu veröffentlichte Liste ließ sich nicht lesen; die ältere gespeicherte Liste wird verwendet. | Bei Dauer freemapper aktualisieren. |
| `warning: the … has no tile(s) … inside Schleswig-Holstein` | Im Downloadportal fehlen einzelne Kacheln. | Die Vorlagen bleiben dort leer. |
| `the Downloadportal lists at most … tiles per request` | Das Gebiet braucht mehr Kacheln, als das Downloadportal je Anfrage nennt. | Das Gebiet kleiner wählen. |
| `the Downloadportal answered with an error page` | Das Downloadportal (Schleswig-Holstein) liefert statt der Kachel eine Fehlerseite. | Später erneut versuchen; freemapper lädt die Kachel einmal selbst erneut. |
| `warning: DGM1 tile … is not in the INSPIRE ATOM directory` / `… does not read (…); used its XYZ from the Downloadportal` | Das GeoTIFF der Geländekachel fehlt im INSPIRE-ATOM-Verzeichnis oder lässt sich nicht lesen (Fehlerseite, anderes Raster, Serverfehler); freemapper hat dieselbe Kachel als XYZ aus dem Downloadportal genommen. Scheitert auch das, nennt der Fehler beide Gründe. | Nichts; die Höhen sind dieselben. |
| `the Downloadportal's tile list … could not be read` | Die Kachelliste ließ sich nicht lesen; die aus einem früheren Lauf wird verwendet. | Nichts, sofern die Daten dieselben sind. |
| `warning: the surface model (bDOM …) is older than the orthophoto (…)` | Das bDOM ist mindestens ein Jahr älter als das Luftbild (in Schleswig-Holstein ab der 2025er Hälfte). | Neubauten im nDOM ggf. von Hand prüfen. |
| `warning: surface model (LVermGeo SH bDOM) unavailable` | Das bDOM ließ sich nicht laden; ein Fehler dabei ist nur eine Warnung. | Die nDOM-Vorlage fehlt; später erneut versuchen. |
| `downloading bDOM tile … (105 MB, … of …)` | Fortschrittsmeldung: freemapper lädt eine ganze Kachel, das dauert etwa 90 s. | Abwarten. |
| `geodaten.schleswig-holstein.de answered with a redirect (HTTP …) to …` | Das Downloadportal leitet auf eine andere Adresse um. freemapper folgt dem nicht, weil es diesen Server nur gegen die eigenen Zertifikate prüft. | Einen Fehler melden. |
| `… certificate …` (Server `geodaten.schleswig-holstein.de`) | Die Zertifikatsprüfung für das Downloadportal schlug fehl, etwa weil dessen Zertifizierungsstelle gewechselt hat. | freemapper aktualisieren, sonst einen Fehler melden. |
| `warning: the map area now lies in another state's data` | Das Gebiet liegt jetzt bei einem anderen Anbieter. | Nichts; freemapper lädt alles neu. |
| `offline and not cached` / `no OSM extract cached` | Mit `--offline` fehlen Daten. | Einmal mit Internet laden. |
| `no tiles fetched yet` / `the map area needs tiles … that were not fetched` | Es wurde noch nicht geladen, oder das Gebiet hat sich geändert. | `run` bzw. `fetch` ausführen. |
| `map already exists (it may contain your work)` | Es gibt schon eine Karte. | `--force` nur benutzen, wenn sie ersetzt werden soll. |
| `warning: … changed upstream` | Der Anbieter hat neue Daten veröffentlicht. | Nichts; die Vorlagen sind jetzt aktuell. |

## 15. Namensnennung

Die Geodaten stehen unter offenen Lizenzen, verlangen aber eine Namensnennung. freemapper schreibt sie
in die Randangaben, in die Kartennotizen und in `ATTRIBUTION.txt`, z. B.:

- `© OpenStreetMap contributors, ODbL`
- `© GeoBasis-DE/LGB <Jahr>, dl-de/by-2-0, Daten geändert` (Brandenburg; in Berlin mit dem Zusatz
  `© Geoportal Berlin, dl-de/zero-2-0`)
- `Quelle: GeoSN, dl-de/by-2-0, Daten geändert` (Sachsen)
- `© GeoBasis-DE / LVermGeo ST <Jahr>, dl-de/by-2-0, Daten geändert` (Sachsen-Anhalt)
- `Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV) <Jahr>, dl-de/by-2-0, Daten geändert` (Hamburg)
- `Geobasisdaten © Hessische Verwaltung für Bodenmanagement und Geoinformation (HVBG) <Jahr>, dl-de/zero-2-0, verändert` (Hessen)
- `© GeoBasis-DE/LVermGeo SH/CC BY 4.0 (Quelle verändert)` und der Lizenzlink https://creativecommons.org/licenses/by/4.0/ (Schleswig-Holstein)
- `Erstellt mit freemapper – https://github.com/HenryJobst/freemapper`

Lassen Sie diese Zeilen auf gedruckten und veröffentlichten Karten stehen.

## 16. Hilfe und Fehler melden

Fragen, Fehler und Wünsche bitte als [Issue auf GitHub](https://github.com/HenryJobst/freemapper/issues).
Hilfreich sind:

- die Meldung;
- der Ort (Name oder Koordinaten);
- die Datei `freemapper.toml` des Projekts.
