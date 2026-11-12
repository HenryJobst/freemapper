# Karte und Trainingskarte im Vergleich

[English](compare.en.md) · [Beispielübersicht](README.md)

freemapper schreibt auf Wunsch zwei PDFs. `--pdf` erzeugt `<name>.pdf`, die Karte mit genau den Objekten der
OOM-Datei, also ohne Kandidaten. `--pdf-candidates` erzeugt `<name>-candidates.pdf`, die Trainingskarte: die Karte
plus jeder Kandidat als Kartenobjekt. Kandidaten sind unsicher abgeleitete Objekte, etwa Höhenlinien, Bäume,
Privatgrundstücke oder Treppen. Das Handbuch erklärt, [was im Gelände zu prüfen ist](../docs/manual/de.md#9-was-im-gelände-zu-prüfen-ist),
und beschreibt die [PDF-Schalter](../docs/manual/de.md#10-karte-als-pdf).

Alle zwölf Beispiele haben die Trainingskarte als PDF. Zwei davon, Cochem und Blankenese, liegen zusätzlich als Karte
allein bei; bei den anderen entsteht sie mit `freemapper run --pdf <Projektordner>`. Die Bilder sind Vorschauen, der
Link unter dem Bild öffnet das PDF.

## Warum die Karte allein so schlicht ist

Was in die Karte kommt, bestimmt das [Inhaltsprofil](../docs/manual/de.md#was-direkt-auf-die-karte-kommt). Zehn der
zwölf Beispiele, auch Cochem, sind entstanden, bevor es die Profile gab; sie entsprechen dem Profil `safe`. Damit
legt freemapper nur verlässliche Registerdaten in die Karte: Gebäude, Straßen, Gewässer, Wald, offenes Land und die
Grundfläche. Alles, was automatisch abgeleitet oder unsicher ist, bleibt Kandidat, damit ein Kartierer es prüft. Dazu
gehören Höhenlinien, Bäume, Zäune und Mauern, Hecken, Sperrflächen und Kontrollstandorte. In Cochem stehen 1315
Kartenobjekte (1156 davon Gebäude) 894 Kandidaten gegenüber: 234 Büsche und Bäume (418), 205 Sperrflächen (520),
151 Höhenlinien (101), 78 Kontrollstandorte (531) und 46 unbefestigte Wege (505.1). Die Karte allein ist deshalb
sicher, aber dünn. Die Trainingskarte zeigt, was freemapper ableiten kann, und jedes dieser Objekte prüfen Sie im
Gelände, bevor Sie es übernehmen.

Heute ist `balanced` das Standardprofil. Damit kommen auch Höhenlinien, Treppen, Wege, Mauern, Zäune und Bäume in
die Karte, und Kandidaten bleiben vor allem die Sperrflächen. Blankenese unten ist so gebaut; Saarlouis auch.

## Cochem

| Karte (Profil `safe`) | mit Kandidaten |
|---|---|
| ![Cochem, nur Kartenobjekte](rheinland-pfalz-cochem/preview-map.webp) | ![Cochem mit Kandidaten](rheinland-pfalz-cochem/preview.webp) |
| [rheinland-pfalz-cochem.pdf](rheinland-pfalz-cochem/rheinland-pfalz-cochem.pdf) | [rheinland-pfalz-cochem-candidates.pdf](rheinland-pfalz-cochem/rheinland-pfalz-cochem-candidates.pdf) |

Die Karte hat 1315 Objekte, vor allem Gebäude (1156), dazu Wasser, Überdachungen und die Grundfläche. Dazu kommen
894 Kandidaten, also 0,7 je Kartenobjekt. 463 stammen aus OSM, 221 aus der ALKIS-Nutzung, 193 aus dem Geländemodell
und 17 aus ALKIS.

Die fünf häufigsten Symbole der Kandidaten:

| Symbol | Anzahl |
|---|---:|
| 418 Busch oder kleiner Baum | 234 |
| 520 verbotenes Gebiet | 205 |
| 101 Höhenlinie | 151 |
| 531 Kontrollstandort | 78 |
| 505.1 unbefestigter Weg | 46 |

Dazu kommen 42 Zählkurven (102). Zusammen machen die Höhenlinien den größten Unterschied im Bild: Sie legen die
Burgberge und Talhänge über das ganze Blatt. Die Bäume und die Sperrflächen füllen den Rest.

Die Karte allein zeigt nur die Stadt, erst die Kandidaten zeigen, wo es steil wird. Sie übernehmen die
Höhenlinien, wenn sie zum Gelände passen, und prüfen jede 520-Fläche vor Ort, bevor sie als Sperrfläche gilt. Von den
Bäumen behalten Sie die markanten, die Kontrollstandorte (531) sind Vorschläge für die Bahnlegung und gehören nicht
in die Karte.

## Blankenese

| Karte (Profil `balanced`, Standard) | mit Kandidaten |
|---|---|
| ![Blankenese, nur Kartenobjekte](hamburg-blankenese/preview-map.webp) | ![Blankenese mit Kandidaten](hamburg-blankenese/preview.webp) |
| [hamburg-blankenese.pdf](hamburg-blankenese/hamburg-blankenese.pdf) | [hamburg-blankenese-candidates.pdf](hamburg-blankenese/hamburg-blankenese-candidates.pdf) |

Blankenese wurde am 12. November 2026 mit dem Standardprofil `balanced` neu gebaut. Die Karte hat 1780 Objekte:
880 Gebäude, 409 Höhenlinien (83 davon Zählkurven), 110 Treppen aus OpenStreetMap (532), dazu Wege, Kanten
befestigter Flächen, Bäume und Zäune. Es bleiben 755 Kandidaten, 0,4 je Kartenobjekt. 688 stammen aus der
ALKIS-Nutzung, 60 aus OSM, 5 aus ALKIS und 2 aus der Generalisierung.

Die fünf häufigsten Symbole der Kandidaten:

| Symbol | Anzahl |
|---|---:|
| 520 verbotenes Gebiet | 696 |
| 531 Kontrollstandort | 32 |
| 401 offenes Land | 9 |
| 418 Busch oder kleiner Baum | 8 |
| 522 Überdachung | 5 |

Neun von zehn Kandidaten sind Sperrflächen. 657 davon sind Wohnbauflächen und 21 gemischt genutzte Flächen aus der
ALKIS-Nutzung, also die Grundstücke zwischen den Treppenwegen. Sie färben die Trainingskarte olivgrün. Welche
Grundstücke wirklich verboten sind, entscheiden Sie im Gelände. Die Höhenlinien des Elbhangs und die Treppen, die
dem Viertel den Namen geben, stehen jetzt schon in der Karte; prüfen müssen Sie sie trotzdem.

## Wie die beiden Beispiele gewählt wurden

Die Wahl folgt einer Messung an den elf Beispielen, die es damals gab, alle im Stand vor den Inhaltsprofilen. Jedes
wurde als Karte und als Trainingskarte gerendert (A3, 120 dpi). Die mittlere Pixeldifferenz ist der Mittelwert der Farbabweichung über das ganze Blatt in Prozent des
Wertebereichs. Die zweite Spalte nennt den Anteil der Pixel, die sich sichtbar unterscheiden (Abweichung über
24 von 255 in einem Farbkanal). Das Blatt enthält auch den weißen Titelblock, deshalb sind die absoluten Werte klein;
es zählt die Reihenfolge. Die Tabelle ist nach der Pixeldifferenz sortiert.

| Beispiel | Kartenobjekte | Kandidaten | Kandidaten je Kartenobjekt | mittlere Pixeldifferenz | geänderte Pixel |
|---|---:|---:|---:|---:|---:|
| [Cochem](rheinland-pfalz-cochem/) | 1315 | 894 | 0,7 | 4,9 % | 16,8 % |
| Blankenese (alter Stand, `safe`) | 969 | 979 | 1,0 | 4,9 % | 16,3 % |
| [Monschau](nordrhein-westfalen-monschau/) | 993 | 851 | 0,9 | 4,6 % | 16,1 % |
| [Marburg](hessen-marburg/) | 1541 | 1065 | 0,7 | 3,9 % | 14,9 % |
| [Bautzen](sachsen-bautzen/) | 1226 | 1376 | 1,1 | 3,9 % | 14,9 % |
| [Viktoriapark](berlin-viktoriapark/) | 766 | 2263 | 3,0 | 3,7 % | 13,9 % |
| [Quedlinburg](sachsen-anhalt-quedlinburg/) | 1642 | 1111 | 0,7 | 3,7 % | 13,2 % |
| [Sanssouci](brandenburg-sanssouci/) | 443 | 1981 | 4,5 | 3,5 % | 15,2 % |
| [Wismar](mecklenburg-vorpommern-wismar/) | 2486 | 1757 | 0,7 | 2,5 % | 9,4 % |
| [Herrenhausen](niedersachsen-herrenhausen/) | 275 | 2142 | 7,8 | 2,1 % | 9,3 % |
| [Lübeck](schleswig-holstein-luebeck/) | 2292 | 1766 | 0,8 | 1,8 % | 6,9 % |

Die Erwartung aus den Zählungen war eine andere: Sanssouci und Herrenhausen haben mit 4,5 und 7,8 die meisten
Kandidaten je Kartenobjekt. Ihre Karten sind aber klein, und die Kandidaten sind zu großen Teilen Bäume, kleine Punkte, die
wenig Fläche ändern. In Cochem und Blankenese ändern Höhenlinien und Sperrflächen große Teile des Blattes. Monschau
liegt mit 4,6 % dicht dahinter, der Abstand zu den ersten beiden ist klein. Die ersten drei Plätze blieben bei 60 dpi
und 120 dpi gleich.

Mit `balanced` rücken Karte und Trainingskarte zusammen, weil die Höhenlinien schon in der Karte stehen. Dieselbe
Messung ergibt für das neu gebaute [Blankenese](hamburg-blankenese/) 2,9 % und 11,3 % (1780 Kartenobjekte, 755
Kandidaten) und für [Saarlouis](saarland-saarlouis/) 2,2 % und 9,4 % (1465 Kartenobjekte, 249 Kandidaten).

Die Zählung der Symbole stammt aus `templates/candidates.omap` des jeweiligen Beispiels. Die Trainingskarte lässt
Punkte unter einer 520-Fläche und nicht erreichbare Durchgänge weg (`freemapper:hidden`), deshalb deckt sich das Bild
nicht in jedem Punkt mit dieser Zählung.

Erstellt mit freemapper – https://github.com/HenryJobst/freemapper
