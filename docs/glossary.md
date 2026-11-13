# Glossar / Glossary

Verbindliche Begriffe für Code und Dokumentation. Neue Fachbegriffe werden **zuerst hier** ergänzt,
dann verwendet. Code-Bezeichner sind Englisch (Rust-Konventionen: `TypeName`, `fn_name`, `CONST_NAME`).

Binding terms for code and documentation. New domain terms are added **here first**, then used.
Code identifiers are English (Rust conventions: `TypeName`, `fn_name`, `CONST_NAME`).

Abkürzungen der Normen / standard abbreviations: **ISSprOM** = International Specification for Sprint
Orienteering Maps (IOF, 2019-2), **ISOM** = International Specification for Orienteering Maps (IOF, 2017-2), **IOF-Schulsymbolsatz / IOF school symbol set** = Symbol Set for School
Orienteering Maps 2019 (IOF).

## Orientierungslauf & Kartografie / Orienteering & cartography

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Orientierungslauf (OL) | orienteering | Laufsportart mit Karte und Kompass. | – |
| OL-Karte | orienteering map | Karte nach IOF-Norm (ISOM/ISSprOM). | `Map` |
| Sprintkarte | sprint map | OL-Karte nach ISSprOM, typisch 1:4000 oder 1:3000, städtisches Gelände. | `MapKind::Sprint` |
| Waldkarte | forest map | OL-Karte nach ISOM 2017-2 für Wald, 1:15 000 (Symbole 100 %), 1:10 000 (150 %) oder 1:7500 (200 %). Wege aus OSM als Kartenobjekte, Wald als weißes Papier, Höhenlinien alle 5 m oder in flachem Gelände 2,5 m. Die ISSprOM-Codes der Bausteine werden über `isom-aliases.toml` zu ISOM-Codes. | `MapKind::Forest`, `SymbolSet::isom_2017_2` |
| Schulhofkarte | schoolyard map | OL-Karte für Schulgelände nach „IOF Symbol Set for School Orienteering Maps 2019“, Maßstab 1:1000, ISSprOM-Symbole auf 125 % vergrößert plus 6 Zusatzsymbole. | `MapKind::Schoolyard` |
| Waldkarte | forest map | OL-Karte nach ISOM, typisch 1:15000/1:10000. | `MapKind::Forest` |
| Kartenart | map kind | Sprint- oder Waldkarte; bestimmt Norm, Symbolsatz und Maßstab. | `MapKind` |
| Kartierer | mapper | Person, die die OL-Karte zeichnet und im Gelände prüft. | – |
| Kartengebiet | map area | Vom Nutzer gewähltes Polygon, für das die Karte erzeugt wird. | `MapArea` |
| Puffer | buffer | Zusätzlicher Rand um das Kartengebiet beim Laden und Ableiten. | `buffer_m` |
| Begrenzungsrahmen | bounding box (bbox) | Achsparalleles Rechteck um eine Geometrie. | `BoundingBox` |
| Maßstab | scale | Verhältnis Karte : Natur, gespeichert als Maßstabszahl (z. B. 4000). | `scale_denominator` |
| Symbol | symbol | Kartenzeichen einer Norm (Punkt-, Linien-, Flächen-, Text- oder kombiniertes Symbol). | `Symbol` |
| Symbolnummer | symbol code | Nummer des Symbols in der Norm, z. B. „521“ (Gebäude), „501.5“. | `SymbolCode` |
| Symbolsatz | symbol set | Sammlung aller Symbole einer Norm in einem Maßstab, z. B. „ISSprOM 2019“ in OOM. | `SymbolSet` |
| Symbolvergrößerung | symbol enlargement | Faktor, um den alle Symbolmaße vergrößert werden (Schulkarte: 1,25). | `symbol_enlargement` |
| Symbolmaß | symbol measure | Länge eines Symbols, wie die Norm es bei 100 % zeichnet, in mm. Wächst auf dem Blatt mit der Symbolvergrößerung, auch die Mindestgrößen der Norm (kleinstes Gebäude 0,5 mm: auf der Schulkarte 0,625 mm); lässt sich nur mit Maßstab und Vergrößerung (`SymbolScale`) in Geländemeter oder Kartenkoordinaten umrechnen (ADR 0118). | `SymbolMm`, `SymbolMm2`, `build::symbol_measures` |
| Papiermaß | paper measure | Länge auf dem gedruckten Blatt in mm, unabhängig von der Symbolvergrößerung, z. B. eine Vereinfachungstoleranz oder die Maßstabsleiste. Mindestgrößen der Norm sind Symbolmaße (ADR 0118). | `PaperMm`, `PaperMm2` |
| Symbolzuordnung | symbol mapping | Regel, welche Quelldaten zu welchem Symbol werden. | `SymbolMapping` |
| Kartenobjekt | map object | Einzelnes gezeichnetes Element mit Symbol und Geometrie. | `MapObject` |
| Kartenteil | map part | Gruppe von Kartenobjekten in OOM (Ebene). | `MapPart` |
| Kartenvorlage | template | Hintergrunddatei (Raster oder Vektor) in OOM, die nicht Teil der Karte ist und zum Abzeichnen dient. | `Template` |
| Vorlagenkarte | template map | OOM-Datei, die in einer anderen OOM-Datei als Kartenvorlage eingebunden ist. | `TemplateMap` |
| Kandidat | candidate | Unsicher abgeleitetes Objekt, das nur als Vorlage geliefert wird (z. B. Privatgelände, Barriere). | `Candidate` |
| Inhaltsprofil | content profile | Voreinstellung dafür, was direkt auf die Karte kommt: `safe`, `balanced` (Standard), `full` (ADR 0100). | `Profile` |
| Objektgruppe | object group | Gruppe von Objekten, die der Nutzer auf die Karte, zu den Kandidaten oder weglassen schicken kann (Mauern, Zäune, Hecken …). | `Group`, `Place` |
| Startprojekt | starter project | Erstes, von freemapper erzeugtes OOM-Projekt, ab dem in OOM kartiert wird. | `StarterProject` |
| Abgleich | reconciliation | (später) Vergleich neuer Quelldaten mit einer bereits bearbeiteten Karte. | `reconcile` |
| Generalisierung | generalization | Vereinfachen von Geometrie auf Kartenmaßstab (Mindestgrößen, Vereinfachung). | `generalize` |
| Punktgeneralisierung | point symbol generalization | Die Punktsymbole, die freemapper erzeugt (Kandidaten und Kartenobjekte, beide Kartenarten): in einem 520-Kartenobjekt entfernt, unter einem 520-Kandidaten als Kandidat behalten mit `freemapper:hidden=520` und nur in der Trainingskarte weggelassen, eine dichte Baumgruppe wird zur Gruppenfläche, sonst entfällt bei zu kleinem Mindestabstand das kleinere Symbol. Der behaltene Punkt nennt die entfallenen in `freemapper:generalized`. | `generalize_points`, `build/points.rs` |
| Mindestabstand (Punktsymbole) | minimum gap (point symbols) | Abstand zwischen den Außenkanten zweier Punktsymbole auf Papier: 0,15 mm × Symbolvergrößerung (ISSprOM §3.3, dort für Linien gleicher Farbe; für Punkte gesetzt). Symbolgröße = größeres aus Außenmaß der Norm und gezeichnetem Symbol. | `POINT_GAP_MM`, `POINT_SIZES_MM` |
| Baumgruppe | tree group | Mindestens 3 Bäume, verkettet (Einzelverkettung), wenn sie einander den Mindestabstand nicht lassen, näher als 2,5 mm auf dem Papier stehen oder ihre Kronen sich bis auf 0,5 mm nähern. Eine Reihe (Korridor schmaler als 2 Symbolgrößen, oder mindestens 3-mal so lang wie breit und schmaler als 2 Kronen; auf Pflaster jede so lange Gruppe) bleibt Punkte und wird ausgedünnt. Sonst beansprucht jeder Baum einen Kreis von 1,25 mm Halbmesser oder seine Krone; die Vereinigung, zu 0,5 mm geschlossen und um mindestens 1,2 Anspruchsbreiten geöffnet, wird zur Fläche: 501.3, wenn mindestens die Hälfte auf Pflaster liegt (dort nur die Kronen, mindestens 9 Bäume); sonst 405 ab 50 % Kronendeckung, auf weißem Grund (weniger als die Hälfte auf 401 bis 404) mit einer 416-Grenze, sonst 402 (dort bleiben die 417 Punkte). Unter der Mindestgröße entfällt die Fläche. `freemapper:source=generalization`. | `GROUP_MIN_TREES`, `GROUP_LINK_MM`, `FOREST_COVER`, `OPEN_LAND_SHARE`, `GROUP_CLOSE_MM`, `is_row` |
| Vegetationsgrenze (416) | distinct vegetation boundary (416) | Punktierte Linie um einen Wald (405) aus einer Baumgruppe auf weißem Grund, damit der weiße Wald sichtbar ist. Nur auf der Schulhofkarte, die keine belaufbare Fläche hat; auf der Sprintkarte zeigt sich der weiße Wald auf Gelb oder Braun ohne Linie. Sie fehlt, wo die Fläche an einem Gebäude, Gewässer, Sperrgebiet, Pflaster oder am Rand des Ausschnitts endet, und ist mindestens 2,65 mm lang. | `BOUNDED_FOREST`, `boundary_lines` |
| Überlappende Linien | overlapping barrier lines | Mauern, Zäune, Hecken und Stützmauern (515, 518, 410.1, 513.2), die parallel (höchstens 30°) näher laufen, als ihre Symbole dürfen (halbe Breiten plus 0,15 mm bei zwei schwarzen Linien, bei zwei Quellen mindestens 1,5 m): Die schwächere Linie (Rang, dann Symbol 515 > 518 > 410.1 > 513.2) verliert das Stück. Gleiche Quelle und gleiches Symbol bleiben. Ein Stück bis 0,5 m parallel neben einem Gebäudeumriss fällt weg, Reststücke unter 1 mm auch. Ein Baum bis 1 m neben einer dieser Linien fällt weg. | `resolve_barrier_overlaps`, `build/barriers.rs` |
| Kronendurchmesser | crown diameter | Durchmesser einer Baumkrone in m (`freemapper:crown_m`): Berliner `kronedurch`, Stadtbaumkataster `tree:crown_m`, OSM `diameter_crown`; ohne Angabe 12 m (417) bzw. 5 m (418). | `DEFAULT_CROWN_M` |
| Verborgener Kandidat (520) | hidden candidate (520) | Punktkandidat, dessen Mitte in einem 520-Kandidaten liegt: Er bleibt in `candidates.omap` und den OCAD-Kandidaten mit `freemapper:hidden=520`, nimmt an Baumgruppen und Ausdünnung nicht teil und fehlt nur in der Trainingskarte. In einem 520-Kartenobjekt entfällt ein Punkt ganz. | `HIDDEN_KEY` |
| Auflösen | dissolve | Benachbarte oder überlappende Flächen zu einer Fläche vereinigen, sodass zwischen ihnen keine Grenzlinie bleibt, z. B. die vielen ALKIS-Flächen eines Flusses. | `dissolve` |
| Glättung | smoothing | Entfernen von Rauschen aus Linien oder Rastern. | `smooth` |
| Namensnennung | attribution | Pflichtvermerk der Datenlizenz auf Karte und in Dateien. | `Attribution` |

## Kartenzeichen (Auswahl ISSprOM 2019) / map features (ISSprOM 2019 selection)

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Höhenlinie | contour | Linie gleicher Höhe (ISSprOM 101). | `Contour` |
| Zählkurve | index contour | Hervorgehobene Höhenlinie, jede fünfte (ISSprOM 102). | `IndexContour` |
| Formlinie | form line | Zwischenhöhenlinie für Geländeformen (ISSprOM 103). | `FormLine` |
| Äquidistanz | contour interval | Höhenabstand benachbarter Höhenlinien in Metern (Sprint: 2 m). | `contour_interval_m` |
| Kleine Kuppe | small knoll | Kleiner, deutlicher Hügel, den die Höhenlinien nicht zeigen (ISSprOM 109); längliche Kuppe 110. Kandidat aus dem DGM: ein kleiner geschlossener Ring feiner Höhenlinien um einen Gipfel. | `LandformKind::Knoll`, `LandformKind::ElongatedKnoll` |
| Kleine Senke | small depression | Kleine, flache Vertiefung (ISSprOM 111). Kandidat aus dem DGM wie die Kuppe, auf dem umgekehrten Gelände. | `LandformKind::Depression` |
| Erdböschung | earth bank | Steiler Geländeabsatz (ISSprOM 104); ab 3 m Höhe aus dem DGM als nicht begehbare Felswand (201) vorgeschlagen. | `LandformKind::Bank`, `LandformKind::Cliff` |
| Geländeform-Kandidat | landform candidate | Kuppe, Senke, Böschung oder Felswand, die `derive` aus dem DGM findet; nur in der Kandidatenkarte. | `Landform`, `data/landforms.geojson` |
| Höhensprung | height jump | Schmaler, langer, steiler Streifen im DGM, der den größten Teil seines Höhenunterschieds auf wenigen Metern abbaut; Grundlage der Böschungs- und Felswand-Kandidaten. | `height_jumps` |
| Grünabstufung | green shade | Vegetation, die das Laufen bremst: langsam belaufbar (406), schwer belaufbar (408), nicht belaufbar (410). Kandidat aus der Punktwolke nach dem Anteil der Rückstrahlungen in 0,8 bis 3 m Höhe. | `VegetationClass::SlowRunning`, `Walk`, `Impassable` |
| Unterwuchs | undergrowth | Gute Sicht, aber gebremstes Laufen unter Bäumen (407, 409). Kandidat aus dem Anteil der Rückstrahlungen in 0,2 bis 0,8 m Höhe. | `VegetationClass::SlowRunningGoodVisibility`, `WalkGoodVisibility` |
| Vegetations-Kandidat | vegetation candidate | Fläche aus der Punktwolke (401, 405 bis 410) in `data/vegetation.geojson`; Gruppe `vegetation`, ohne eigene Wahl aus. | `CellCounts`, `stored_vegetation_objects` |
| Gebäude | building | Gebäudeumriss (ISSprOM 521). | `Building` |
| Überdachung | canopy | Überdachte, begehbare Fläche, z. B. Arkade (ISSprOM 522). Nicht Baumkrone! In Berlin werden die ALKIS-Bauwerke `Überdachung` und `Carport` zu Kartenobjekten. | `Canopy`, `canopy_objects` |
| Treppe | stairway | Treppe als Linie in Laufrichtung (ISSprOM 532.6 bis 532.9); in Berlin Kandidaten aus den ALKIS-Bauwerken `Treppe` und `Freitreppe`, im vollständigen ALKIS aus den Flächen mit `bauwerksfunktion` 1620/1621. | `stair_objects` |
| Laufrichtung (Treppe) | walking direction | Richtung, in der man eine Treppe hinauf- oder hinabgeht. freemapper nimmt die Achse des Treppenumrisses mit dem größeren Höhenunterschied im DGM; steigt keine um 0,3 m, die Achse zu einem angrenzenden Gebäude oder Denkmal oder sonst die kurze Achse, markiert mit `freemapper:direction=guessed`. Hat eine NAS-Treppe eine Treppenunterkante, gibt diese die Richtung. | `STAIR_MIN_RISE_M` |
| Stufenkante | step edge | Kante einer Stufe; in Berlin die `Freitreppe`-Linien der ALKIS-Bauwerke. Parallele Kanten bis 2 m Abstand werden eine Treppe quer zu den Kanten. | `StepEdge`, `STEP_GAP_M` |
| Treppenunterkante | stair bottom edge | NAS-Linie mit `bauwerksfunktion` 1630 am Fuß einer Treppe, über `gehoert_zu` mit ihrer Treppenfläche (1620/1621) verbunden. Die Treppe steigt von ihr weg und ist dann nicht `guessed`. Eine Kante außerhalb ihrer Fläche zählt nicht, eine Kante allein ergibt keine Treppe. | `StairInput::nas`, `axis_from_edge` |
| Breite Treppe | wide stair | Treppe breiter als dreimal 532.9 auf dem Papier (Sprint 12 m), z. B. eine Tribüne: parallele Linien 532.9, Tag `freemapper:size=wide`. | `WIDE_STAIR_FACTOR` |
| Treppenbreite | stair footprint | Breite der Treppe quer zur Laufrichtung. freemapper rechnet sie auf Papier um (Maßstab, Symbolvergrößerung) und wählt das nächste Symbol: 532.6 (0,4 mm, auf 1:4000 1,6 m) bis 532.9 (1 mm, 4 m). | `footprint_code` |
| Belaufbare Fläche | walkable area | Die Grundfläche der Sprintkarte: ein Flächenobjekt im unteren Braun (Farbe „Lower brown 30% for walkable area“, CMYK wie 501.5, Farbpriorität hinter allen anderen Farben) über Kartengebiet und Puffer, im eigenen Kartenteil „walkable area“ zuerst geschrieben, Tag `freemapper:source=base`. Alles andere liegt darüber. Symbol 990 „Walkable area (base fill)“, deutsch „Belaufbare Fläche (Grundfläche)“. Nur Sprintkarte. | `walkable_base`, `WALKABLE_AREA_CODE` |
| Offenes Gelände (Park) | open land (park) | ALKIS-Nutzungen, die zu 401 werden: Grünanlage, Park, Siedlungsgrünfläche und Grünland als Kartenobjekt, Friedhof und das übrige Landwirtschaftsland als Kandidat; Wald und Gehölz werden 405, Heide 403. OSM `leisure=park` und `landuse=grass|meadow|village_green|recreation_ground` sind 401-Kandidaten, soweit kein ALKIS-401 darunter liegt. | `open_land_objects`, `build/open_land.rs` |
| Befestigte Fläche | paved area | Asphalt-, Pflaster- oder Plattenfläche (ISSprOM 501). | `PavedArea` |
| Unbefestigter Weg | unpaved path | Weg/Pfad ohne Befestigung (ISSprOM 505/506/507). | `UnpavedPath` |
| Verkehrsfläche | traffic area | ALKIS-Nutzung `Straßenverkehr`, `Weg`, `Platz`; Quelle der befestigten Flächen der Sprintkarte. | `paved_area_objects` |
| Regelziel | rule target | Wohin eine OSM-Regel ein Objekt schreibt: in die Karte (verlässlich) oder in die Kandidaten. | `Target` |
| Begleitfläche | verge | ALKIS `Begleitfläche Straßenverkehr`: Grünstreifen und Baumreihen neben der Fahrbahn; wird nicht gezeichnet. | `VERGE` |
| Gewässer | body of water | Fluss, Kanal, See, Teich, Hafenbecken oder Meer. Auf beiden Kartenarten werden die ALKIS-Nutzungen `Fließgewässer`, `Stehendes Gewässer`, `Hafenbecken` und `Meer` zu Kartenobjekten 301.1 und 301.6, Gewässer aus OSM zu Kandidaten. | `water_objects`, `WATER_LAND_USES` |
| Unpassierbares Gewässer | uncrossable body of water | Wasser, das nicht durchquert werden darf (ISSprOM 301). freemapper zeichnet ALKIS-Gewässer als Fläche 301.1 mit Uferlinie 301.6 und OSM-Gewässer mit dem kombinierten Symbol 301. | `FILL_CODE`, `OSM_AREA_CODE` |
| Passierbares Gewässer | crossable body of water | Flaches Wasser, das durchquert werden darf (ISSprOM 302). freemapper erzeugt es nicht; der Kartierer stuft flache Gewässer im Gelände um. | – |
| Uferlinie | bank line | Schwarze Randlinie eines unpassierbaren Gewässers (ISSprOM 301.6). Wo ein Gewässer über Kartengebiet plus Puffer hinausreicht, fehlt sie an dieser Kante; Inseln bekommen eine eigene. | `BANK_CODE`, `bank_lines` |
| Mittlere Breite | mean width | 2 · Fläche / Umfang einer Fläche. ALKIS-Gewässer unter 2 m mittlerer Breite werden Kandidaten statt Kartenobjekte. | `mean_width`, `NARROW_WATER_M` |
| Kleiner Wasserlauf | small crossable watercourse | Bach oder Abzugsgraben unter 2 m Breite, der ganzjährig Wasser führt (ISSprOM 305); Kandidat aus OSM `waterway=stream` oder `drain`. Gräben (`ditch`) sind immer 306. | `"305"` |
| Graben | minor or seasonal watercourse | Graben oder Bach, der zeitweise trocken fällt (ISSprOM 306); Kandidat aus OSM `waterway=ditch` und aus `stream` oder `drain` mit `intermittent=yes`. | `"306"` |
| Brunnen | small fountain or well | Brunnen oder Wasserstelle als Punkt (ISSprOM 311); Kandidat aus OSM `amenity=fountain` (Punkt) und `man_made=water_well`. | `"311"` |
| Quelle (Wasser) | spring | Stelle, an der ein Wasserlauf entspringt (ISSprOM 312); Kandidat aus OSM `natural=spring`. | `"312"` |
| Markantes Wasserobjekt | prominent water feature | Auffälliges Wasserobjekt als Punkt (ISSprOM 313); auf Schulhofkarten Kandidat aus OSM `amenity=drinking_water` (Trinkbrunnen). | `"313"` |
| Brücke | bridge | Bauwerk, das Wege über Wasser, Bahn oder Straße führt. ALKIS enthält keine Brücken; OSM-Flächen `man_made=bridge` werden auf Sprintkarten Kandidaten 501.5. Die Uferlinie läuft unter der Brücke durch und muss dort von Hand getrennt werden. | `man_made=bridge` |
| Bahnstrecke | railway | Gleis mit Schienen (ISSprOM 509.1). Aus OSM `railway=rail\|light_rail\|narrow_gauge` (auch Nebengleise, außer Überleitungen) als Kartenobjekt auf Sprintkarten, Brücken eingeschlossen, Tunnel nicht (ADR 0093). | `"509.1"` |
| Straßenbahn | tramway | Gleis der Straßenbahn (ISSprOM 509.2); die Norm zeichnet sie nur, wenn sie der Orientierung dient. Kandidat aus OSM `railway=tram` (ADR 0093). | `"509.2"` |
| Verbotenes Gebiet / Privatgelände | out-of-bounds area | Nicht zu betretende Fläche, olivgrün (ISSprOM 520); Kandidaten aus der ALKIS-Nutzung. | `out_of_bounds_objects` |
| Unpassierbares Objekt / Durchgangsbarriere | impassable feature | Darf nicht überquert werden: Mauer, Zaun, Hecke usw. | `ImpassableFeature` |
| Zaun | fence | Linienhafte Barriere aus Draht/Holz/Metall. | `Fence` |
| Mauer | wall | Linienhafte Barriere aus Stein/Beton. | `Wall` |
| Hecke | hedge | Linienhafte Barriere aus Pflanzen. | `Hedge` |
| Stützmauer | retaining wall | Mauer, die einen Geländesprung abstützt; nur von einer Seite aus hoch (ISSprOM 513.2). In Berlin Kandidat aus ALKIS `Stützmauer*`. | `berlin-candidates.toml` |
| Böschungsoberkante | top of an embankment | Obere Kante einer Böschung, ALKIS-Relief `Oberkante`; in Berlin auf Sprintkarten Kandidat für die Erdböschung (ISSprOM 104). | `berlin-candidates.toml` |
| Durchfahrt | building passage | Weg durch oder unter einem Gebäude (ALKIS-Bauteile `Durchfahrt im Gebäude`, `Durchfahrt an überbautem Verkehrsweg`, `Arkade`); in Berlin Kandidat für 522, das Gebäude bleibt ganz. Aus OSM: `tunnel=building_passage` oder `covered=yes` am Weg, als Streifen 522 (ADR 0072). | `building_part_features` |
| Streifen (Korridor) | corridor | Fläche entlang eines Weges mit fester Breite (Regelfeld `corridor_m`, oder das Tag `width`), Enden gerade abgeschnitten: Durchgang als 522, Privatweg als 520. | `Draw::Corridor` |
| Sperrfläche | area that shall not be entered | ISSprOM 520: Fläche, die nicht betreten werden darf (Privatgelände, Beet, Bahngelände). Aus OSM: `access=private\|no` an Fläche oder Weg (ADR 0073). | `520`, `OUT_OF_BOUNDS_CODE` |
| Fester Kies | firm gravel | `surface=compacted\|fine_gravel`: nach ISSprOM 501 („hard gravel“) befestigt, Kartenobjekt 501.6 bis 501.9 statt 505 (ADR 0073, 0079); im ALKIS-Pflaster geschnitten. | – |
| Kontrollstandort | control site | Kleines künstliches Objekt aus OSM, der Berliner Straßenbefahrung oder dem Beleuchtungskataster (Poller, Laterne, Papierkorb, Fahrradständer, Bank, Mast, Fahnenmast, Werbesäule), das sich als Posten eignet; 531-Kandidat mit dem Tag `freemapper:control`, nie Kartenobjekt, gleichartige innerhalb von 25 m fallen weg (ADR 0074, 0077). | `CONTROL_KEY`, `control_sites` |
| Freileitung | overhead power line | Oberirdische Stromleitung (OSM `power=line\|minor_line`); ISSprOM 510, ab 110 kV 511; Kandidat (ADR 0083). | – |
| Rohrleitung | pipeline | Rohrleitung über der Erde (OSM `man_made=pipeline` mit `location`); niedrig 528, hoch und nicht überquerbar 529; Kandidat (ADR 0083). | – |
| Turm | tower | Hoher Turm oder großer Mast (ISSprOM 524), kleiner Turm (525); aus OSM-Punkten, mit `freemapper:control`; Kandidat (ADR 0083). | `TOWER_CODES` |
| Stolperstein | Stolperstein | Messingplatte im Pflaster zur Erinnerung an Verfolgte; bündig, daher kein auffälliges Objekt und nie 526 (ADR 0075). | – |
| Erreichbarer Durchgang | reachable passage | 522-Durchgang, der (allein oder über eine Kette von Durchgängen) an belaufbaren Grund grenzt, der mit dem Straßennetz zusammenhängt; sonst `freemapper:hidden=unreachable` (ADR 0076). | `hide_unreachable` |
| Bahnleger | course setter | Person, die die Bahnen (Posten und Reihenfolge) eines Orientierungslaufs plant. | – |
| Denkmal | monument | Denkmal, Denkstein oder Standbild als Punkt (ISSprOM 526); ab 3 m Breite mit `freemapper:size=large`. | `berlin-candidates.toml` |
| Großer Baum | prominent large tree | Markanter Baum (ISSprOM 417); in Berlin Bäume des Baumkatasters mit Krone ab 10 m. | `berlin-candidates.toml` |
| Busch oder kleiner Baum | prominent bush or small tree | Markanter Busch oder kleiner Baum (ISSprOM 418); OSM `natural=tree` und die Bäume des Berliner Baumkatasters mit Krone unter 10 m. | `osm-candidates.toml`, `berlin-candidates.toml` |
| Hydrant | hydrant | Hydrant, Handpumpe, markanter Wasserpunkt (Schulsatz 321). | `Hydrant` |
| Spielgerät | playground equipment | Spielgerät oder Handballtor (Schulsatz 541). | `PlaygroundEquipment` |
| Tisch | table | Tischtennis- oder Picknicktisch (Schulsatz 542). | `Table` |
| Bank | bench | Feste Sitzgelegenheit (Schulsatz 543). | `Bench` |
| Mast | pole | Fahnenmast, Schild, Basketballkorb (Schulsatz 544). | `Pole` |
| Laterne | lamppost | Straßenlaterne oder Ampel (Schulsatz 545). | `Lamppost` |
| Tartanbahn | tartan track | Kunststoffbelag von Laufbahn/Sportfeld, im Schulsatz 501 mit 50 % Braun. | `TartanTrack` |
| Randangaben | peripheral information | Maßstab, Äquidistanz, Titel, Maßstabsleiste, Copyright (später Nordbalken, Legende) außerhalb des Kartenbilds, im Kartenteil „peripheral information“. | `PeripheralInfo` |
| Nordlinien | magnetic north lines | Linien nach magnetisch Nord über dem Kartenbild (ISSprOM 601, Abstand 30 mm × Symbolvergrößerung). Kartenobjekte im Kartenteil „default part“, nicht Teil der Randangaben (ADR 0043). | `north_lines` |
| Maßstabsleiste | bar scale | Strecke mit Teilstrichen und Metern im Gelände, Teil der Randangaben. | `bar_scale` |
| Baumkrone | tree crown | Vom Laserscan erfasstes Blätterdach eines Baumes. | `TreeCrown` |

## Geodaten / geodata

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Geobasisdaten | official geodata | Amtliche Vermessungsdaten der Bundesländer (hier: LGB Brandenburg). | – |
| Datenquelle | data source | Anbieter + Produkt, z. B. LGB DGM1, OSM. | `DataSource` |
| Quelladapter | source adapter | Code, der eine Datenquelle findet, lädt und normiert; für ein Bundesland oder Land siehe Regionsadapter. | `SourceAdapter` |
| Regionsadapter | region adapter | Alles, was ein Bundesland oder Land liefert: Grenze, Koordinatensystem, OSM-Auszüge, Quellenangabe, Schätzposten und wahlweise ALKIS, Raster und Zusatzebenen. Die Engine fragt die Adapter der Registry und nennt selbst kein Bundesland; die Projektdatei speichert Regionen und Anbieter als Kennung. | `RegionAdapter`, `Registry`, `AdapterId` |
| Adaptermanifest | adapter manifest | TOML-Datei (Format 1), die die Standarddienste eines Regionsadapters (WMS, WCS, WFS, OGC API Features, Geofabrik-Pfad) deklariert und bei Bedarf ein Rasterbackend nennt. Wird zur Laufzeit aus einem Ordner gelesen und gilt als nicht vertrauenswürdig. | `ManifestAdapter` |
| Rasterbackend | raster backend | Die Rasterquellen eines Anbieters: Es holt Luftbild, Gelände- und Oberflächenmodell eines Kartengebiets und sagt `derive`, wo sie liegen (Kacheln im Zwischenspeicher oder Dateien im Projektordner). Jeder Anbieter hat eines; ein Manifest nennt später eines beim Namen, wenn die Quelle kein Standarddienst ist (Kachelliste, Zip-Bereiche, Metalink, festes Zertifikat). | `RasterBackend`, `ProviderInfo`, `DeriveInputs` |
| Vertrauensanker | trust anchors | Zertifikate, gegen die genau ein Host geprüft wird, weil er eine unvollständige Kette sendet (Downloadportal von LVermGeo SH). Das Rasterbackend des Anbieters nennt Host und Zertifikate; der HTTP-Client folgt für diesen Host keiner Weiterleitung. | `TrustAnchors`, `RasterBackend::trust` |
| Luftbildvorschau | aerial preview | Das Luftbild einer Region für die Karte der Gebietsseite: ein WMS mit Ebene, Format, Quellenangabe und Kurzname der Behörde. Es ist nicht der Dienst, von dem das Orthofoto geholt wird. Die Seite bekommt die Vorschauen über `api/adapters`, und nur ihre Hosts stehen in der Content-Security-Policy. | `PreviewInfo`, `RegionInfo::preview` |
| Umriss | outline | Grenze einer Region in EPSG:25833 mit dem Rechteck darum; an ihn richten sich die Fragen, ob ein Gebiet, ein Punkt oder eine Kachel in der Region liegt. | `Outline`, `boundaries::outline` |
| Adapterprüfung | adapter check | `freemapper adapters check <Ordner>`: prüft den Ordner eines Adapters, ohne einen Server zu fragen (ob er laden würde, Befunde und Warnungen), und mit `--live --bbox` zusätzlich, was seine Dienste für ein kleines Rechteck liefern. | `adapters check`, `check_folder` |
| Produkt | product | Einzelner Datensatztyp einer Quelle, z. B. DGM1, nDOM, DOP20. | `Product` |
| Kachel | tile | Rechteckiges Teilstück eines Produkts (LGB: 1×1 km). | `Tile` |
| Kachelliste | tile list | Liste eines Anbieters mit Namen (und Datum) der verfügbaren Kacheln; bei Geobasis NRW die `*_meta.zip` je Produkt, bei LVermGeo RP die WFS-Ebene `Laserbefliegung` (Flugtage je Kachel, daraus das Jahr im Dateinamen). | `meta_list`, `lvermgeo_rp::tile_list` |
| Befliegungsübersicht (Laserbefliegung) | laser-scan flight index | WFS-Ebene `Laserbefliegung` des LVermGeo RP: je 1-km-Kachel Flugjahr und Flugtage des Laserscans. freemapper bildet daraus den Dateinamen der DGM1- und DOM1-Kachel und nimmt den letzten Flugtag als Datum. | `lvermgeo_rp::TILE_LIST_URL`, `ListedTile` |
| Metalink | Metalink | XML-Datei (`.meta4`), die Namen, Größen und Prüfsummen der Dateien eines Servers nennt. LVermGeo RP veröffentlicht je Produkt eine für das ganze Bundesland (etwa 11 MB); freemapper liest sie nur für Kacheln, denen die Befliegungsübersicht kein Flugjahr gibt. | `lvermgeo_rp::metalink_url`, `parse_metalink` |
| 404-Vermerk | missing-tile note | Datei `<Name>.missing` im Cache: Der Server hat für eine Kachel der Kachelliste mit 404 geantwortet. Spätere Läufe fragen nicht erneut, bei Geobasis NRW bis zur nächsten Ausgabe der Liste, bei LVermGeo RP 30 Tage lang. | `TileStore::marker_path`, `lvermgeo_rp::RECHECK_AFTER` |
| Kachelschema | tile scheme | Regel für Lage und Benennung der Kacheln. | `TileScheme` |
| Kachelkennung | tile id | Name einer Kachel, z. B. `33421-5821` (SW-Ecke in km). | `TileId` |
| Datenstand / Aktualität | data currency | Erfassungs- bzw. Veröffentlichungsdatum eines Datensatzes. | `currency` |
| Herkunft | provenance | Quelle, Quell-ID und Datenstand eines Kartenobjekts. | `Provenance` |
| Quell-ID | source id | Identifikator eines Objekts in der Quelle (ALKIS `oid`, OSM-ID). | `source_id` |
| Zwischenspeicher | cache | Lokaler Speicher heruntergeladener Kacheln, projektübergreifend. | `Cache` |
| Digitales Geländemodell (DGM) | digital terrain model (DTM) | Höhenraster der Erdoberfläche ohne Bewuchs und Bebauung. | `Dtm` |
| Digitales Oberflächenmodell (DOM) | digital surface model (DSM) | Höhenraster inkl. Bewuchs und Bebauung. | `Dsm` |
| Bildbasiertes DOM (bDOM) | image-based DSM | Aus Luftbildern berechnetes DOM. | `ImageDsm` |
| Normalisiertes DOM (nDOM) | normalized DSM (nDSM) | DOM minus DGM = Objekthöhe über Grund. | `Ndsm` |
| Schummerung | hillshade | Schattiertes Reliefbild aus dem DGM. | `Hillshade` |
| Hangneigung | slope | Neigung des Geländes je Rasterzelle (Grad). | `Slope` |
| Lokales Relief | local relief | DGM minus sein geglättetes Mittel (Sprint 5 m, Schulhof 3 m): Böschungen, Kuppen, Mulden und Gräben treten hervor, das allgemeine Gefälle nicht. Vorlage `templates/relief.png`. | `local_relief`, `RELIEF_RELPATH` |
| Punktwolke | point cloud | Menge von 3D-Messpunkten (LAS/LAZ). | `PointCloud` |
| Laserscan / ALS | airborne laser scanning (ALS) | Flugzeuggestützte LiDAR-Vermessung. | `Als` |
| Bodenpunkt | ground point | Punkt der Klasse „Boden“ in einer Punktwolke. | `GroundPoint` |
| Digitales Orthophoto (DOP) | orthophoto | Entzerrtes, georeferenziertes Luftbild (DOP20 = 0,2 m). | `Orthophoto` |
| Flugdatum | flight date | Tag, an dem das Luftbild beflogen wurde. | `WmsRecord::flight` |
| Erstellungsdatum | date of creation | Feld `erstellung` des Luftbild-Kartendienstes von LVermGeo RP, ein Datum je 2-km-Block. freemapper nimmt es als Flugdatum (ein Schluss aus den Daten, kein Feld heißt „Bildflugdatum“) und vermerkt im Projekteintrag „Aktualität (Erstellung)“. | `lvermgeo_rp::FLIGHT_LABEL`, `DateLabel`, `WmsRecord::flight_label` |
| Belaubung (belaubt / unbelaubt) | leaf state (leaf-on / leaf-off) | Ob die Bäume beim Bildflug Laub trugen; in Niedersachsen nach dem Flugmonat bestimmt (Mai bis Oktober belaubt). | `lgln::leaf_state` |
| Kartendienst (WMS) | map service (WMS) | OGC Web Map Service: liefert ein Bild für einen Ausschnitt; freemapper setzt das Luftbild aus Stücken von höchstens 4000 px zusammen. | `WmsService` |
| WCS-Dienst | WCS service | OGC Web Coverage Service: liefert die Rasterwerte für einen Ausschnitt, z. B. DGM1 und DOM1 in Sachsen-Anhalt, Mecklenburg-Vorpommern und Niedersachsen. | `WcsService` |
| STAC-Suche | STAC search | Abfrage im SpatioTemporal Asset Catalog eines Anbieters, die zu einem Ort die Kacheln mit Datum nennt (LGLN: ein Server je Produkt). freemapper nimmt den neuesten Eintrag in der Mitte des Kartengebiets. | `lgln::newest_item`, `StacItem` |
| OSM-Auszug | OSM extract | Regionaler Abzug der OSM-Daten als PBF-Datei (Geofabrik, z. B. Brandenburg). | `Extract` |
| Regierungsbezirk-Auszug | government district extract | OSM-Auszug eines Regierungsbezirks (Nordrhein-Westfalen: Arnsberg, Detmold, Düsseldorf, Köln, Münster) statt des ganzen Landes; gewählt nach der mitgelieferten Auszugsgrenze, an einer Bezirksgrenze mehrere. | `ExtractRef`, `extracts_for`, `RegionAdapter::extract_boundary` |
| Weltdatei | world file | Textdatei mit Pixelgröße und Lage eines Rasterbilds (`.jgw`, `.pgw`, `.tfw`). | `GridSpec::from_world_file` |
| Mosaik | mosaic | Zusammengesetzte Kacheln, auf Kartengebiet plus Puffer zugeschnitten. | `mosaic_dtm` |
| Geländemosaik | terrain mosaic | Das DGM-Mosaik eines Kachelanbieters, das `derive` unter `data/terrain.tif` speichert, wenn eine Berlin-Zusatzebene Treppenflächen enthält oder eine NAS-Treppe keine Treppenunterkante hat, damit der Export die Laufrichtung der Treppen offline bestimmen kann. | `TERRAIN_MOSAIC_RELPATH` |
| Rasterzelle | cell | Ein Pixel eines georeferenzierten Rasters. | `Cell` |
| Auflösung | resolution | Kantenlänge einer Rasterzelle in Metern. | `resolution_m` |
| Liegenschaftskataster (ALKIS) | cadastre (ALKIS) | Amtliches Kataster: Flurstücke, Gebäude, Nutzung. | `Alkis` |
| Flurstück | parcel | Katasterfläche mit Eigentumsgrenze. | `Parcel` |
| Tatsächliche Nutzung | actual land use | ALKIS-Klassifikation der Flächennutzung (`nutzart`). | `LandUse` |
| ALKIS vereinfacht | simplified ALKIS | Vereinfachtes ALKIS-Schema der Bundesländer (Gebäude/Bauwerke, Nutzung; WFS oder OGC API): Quelle der Gebäude und der Nutzung. | `alkis_wfs`, `WfsService` |
| Vollständiges ALKIS (NAS-konformer WFS) | full ALKIS (NAS WFS) | WFS eines Bundeslandes mit dem vollständigen AAA-Fachschema (GML 3.2, Namensraum `adv:`), z. B. LGB `alkis_nas_wfs`. Liefert vermessene Linien und Punkte (Mauern, Zäune, Hecken, Böschungskanten, Brücken, Denkmäler) für Kandidaten. Ein Eintrag der Diensttabelle je Bundesland. | `NasService`, `NAS_SERVICES`, `LGB_NAS` |
| NAS | NAS (standards-based exchange interface) | Normbasierte Austauschschnittstelle der AdV: das GML-Format des vollständigen ALKIS (Kurven mit Kreisbögen, Flächen aus Kurvenringen, Punkte). | `parse_nas_gml` |
| AdV-Objektart | AdV feature type | Klasse des AAA-Fachschemas, z. B. `AX_SonstigesBauwerkOderSonstigeEinrichtung`; bundesweit gleich, ebenso ihre Codelisten (`bauwerksfunktion` 1740 = Zaun). Steht im gespeicherten Objekt als `nas_type`. | `NasLayer::type_name`, `nas_type` |
| AdV-Codeliste | AdV code list | Bundesweite Liste der Codes einer Eigenschaft einer AdV-Objektart mit deutscher Bezeichnung (GDI-DE-Registry), z. B. `bauwerksfunktion` 1701 = „Mauerkante, rechts“. Die Landesregeln vergleichen Codes; die Bezeichnung steht nur für den Kartierer in `freemapper:alkis`. | `assets/rules/adv-codes.toml`, `nas_label` |
| Bauwerksfunktion | building function code | Eigenschaft `bauwerksfunktion` der AdV-Objektarten `AX_SonstigesBauwerkOderSonstigeEinrichtung`, `AX_BauwerkImVerkehrsbereich` und `AX_BauwerkImGewaesserbereich`, ein Code der AdV-Codeliste, z. B. 1740 Zaun, 1701 Mauerkante rechts, 1620 Treppe, 1800 Brücke. Steht am Kandidaten als `alkis:bauwerksfunktion`. | `alkis:bauwerksfunktion` |
| Landesregeln | state rules | Regeldatei `state-candidates.toml`: macht aus ALKIS-Objekten der Bundesländer (vereinfacht und vollständig; Tags `alkis:<Eigenschaft>`, `alkis:type`, `alkis:geom`) Kandidaten, über dieselbe Regelmaschine wie die OSM-Regeln. Flächenregeln verlangen `alkis:geom=area`. | `Rules::states`, `nas_objects` |
| Punkt auf der Fläche | point on surface | Punkt, der sicher in einer Fläche liegt (anders als der Schwerpunkt einer gebogenen Fläche): der Punkt einer NAS-Fläche für eine Punktregel, z. B. ein Denkmal als 526. | `PointOn::Surface` |
| NAS-Ebene | NAS layer | Eine AdV-Objektart, für bestimmte Kartenarten geladen und unter `data/alkis-nas/<Name>.geojson` im KRS der Karte gespeichert; nur für Gebiete in einem Bundesland mit NAS-Dienst. Jedes Objekt behält `oid` (= `gml:id`), `nas_type`, seine Codes als Text, `beginnt` und `gehoert_zu`. | `NasLayer`, `NAS_LAYERS`, `nas_layers` |
| Gepaarte Mauerkanten | paired wall edges | Zwei NAS-Mauerkanten (1701 rechts: Mauer links der Linie; 1702 links: Mauer rechts; oder die Ringe einer 1700-Fläche), die einander zugewandt sind, fast parallel laufen (höchstens 20°) und auf dem Papier näher als 0,55 mm liegen (Linienbreite 515 plus Abstand, × Symbolvergrößerung). Sie werden, wo beide überlappen, eine Mittellinie 515 mit `freemapper:alkis=Mauer (Mauerkanten gepaart)`; die andere Kante steht in `freemapper:also`. Gepaart wird nur gegenseitig und vor dem Verbinden der Teilstücke; eine vorhandene Mauermitte (1700-Linie, 1703) bleibt, die Kanten daneben entfallen. | `pair_wall_edges`, `WALL_PAIR_MM`, `build/walls.rs` |
| Objektbeginn | object version start | `lebenszeitintervall/beginnt` eines NAS-Objekts: Datum, seit dem diese Version im Kataster steht (kein Vermessungsdatum). Datenstand der NAS-Kandidaten; ohne ihn das Ladedatum. | `beginnt` |
| Kappungsgrenze (NAS) | count cap (NAS) | Höchstzahl Objekte je GetFeature (`COUNT`, LGB 100 000); die NAS-Dienste blättern nicht. Ist eine Antwort gekappt, wird das Gebiet geviertelt, höchstens 4 Stufen tief. | `NasService::max_count`, `NAS_SPLIT_MAX_DEPTH` |
| Kreisbogen (GML) | arc (GML) | Kurvenstück durch drei Punkte (`gml:Arc`, `ArcString`, `Circle`); wird in Schritten von höchstens 10° zu einer Linie verdichtet. | `ARC_STEP_DEG` |
| Landesebenen | state layers | Optionaler Teil des Ladens: vollständiges ALKIS und Baumkataster der Bundesländer außer Berlin. Fehlt der Platz, wird er mit einer Warnung ausgelassen, gespeicherte Ebenen bleiben. | `Part::StateLayers` |
| Berlin-Zusatzebene | extra Berlin layer | Weitere WFS-Ebene des Geoportals Berlin (ALKIS-Bauwerke, Relief, Hecken, Baumbestand, Straßenbefahrung, Register, Gebäudehöhen), unter `data/berlin/<Name>.geojson` gespeichert und nur für Gebiete bei Berlin geladen. Eine Datei kann mehrere Objekttypen enthalten, jeder mit eigenen Kartenarten. | `ExtraLayer`, `EXTRA_LAYERS`, `LayerType` |
| Objekttyp (Geoportal Berlin) | feature type | WFS-Typname ohne Namensraum, z. B. `bd_bordstein`. Steht in der gespeicherten Datei als `freemapper:layer` und am Kandidaten als Tag `berlin:layer`; bei den Ebenen von Berlin plus 1 steht dort der Dateiname. | `LayerType`, `TYPE_KEY` |
| Bauart (Bauteil) | part kind (building part) | Art eines ALKIS-Bauteils, z. B. „Durchfahrt im Gebäude“, „Arkade“ (`bezbat`). | `bauart` |
| Berlin-Regeln | Berlin rules | Regeldatei `berlin-candidates.toml`: macht aus Geoportal-Berlin-Objekten (Tags `berlin:<Eigenschaft>`) Kandidaten, über dieselbe Regelmaschine wie die OSM-Regeln. | `Rules::berlin`, `berlin_objects` |
| Baumkataster | tree cadastre | Die Baumdatenbank des Landes Berlin (Straßenbäume, Anlagenbäume) mit Art und Kronendurchmesser; Bäume mit Krone ab 10 m werden zu 417, die übrigen zu 418. | `strassenbaeume`, `anlagenbaeume`, `berlin-candidates.toml` |
| Stadtbaumkataster | city tree cadastre | Die Baumdatenbank einer Stadt (Potsdam zuerst), je Stadt eine Zeile in `TREE_DATASETS` mit Dienstart, Feldnamen, Ausdehnung und Lizenzzeile. Die Bäume liegen unter `data/trees/<Stadt>.geojson` im KRS der Karte mit `tree_*`-Eigenschaften; jeder Baum behält sein eigenes Aufnahmedatum als Aktualität. Baumstümpfe werden nicht gespeichert. Ohne Kronendurchmesser (Potsdam) sind alle Bäume 418. | `TreeDataset`, `TREE_DATASETS`, `tree_layers`, `trees/…` |
| Bäume der Stadtkarte | city map tree points | Die Baumpunkte der Potsdamer Stadtkarte, drei ODS-Datensätze (`baeume-1-stadtkarte-potsdam`, `-2-`, `-3-`, dl-de/zero-2-0) mit Art (`ggartbez`), Punktnummer (`pktnr`) und Aufnahmedatum (`datum`, 1993 bis 2026), ohne Kronendurchmesser. Alle Bäume werden 418, Baumstümpfe entfallen. | `POTSDAM`, `trees/potsdam` |
| Straßenbefahrung | street survey | Befahrung des öffentlichen Straßenraums Berlins 2014 bis 2016 (WFS `strassenbefahrung`): Flächen, Linien und Punkte mit `gis_id`, das das Aufnahmedatum enthält. Ersetzt ALKIS nie. | `SURVEY_SERVICE`, `survey_date` |
| Aufnahmedatum | survey date | Datum der Straßenbefahrung eines Objekts, aus `gis_id` gelesen (vier Schreibweisen); ohne lesbares Datum 2014-01-01 mit `survey_date_guessed=yes`. | `survey_date`, `stamp_survey_date` |
| Beleuchtungskataster | lighting cadastre | Öffentliche Beleuchtung Berlins (WFS `beleuchtung`); Leuchten außer Betrieb werden beim Laden verworfen. Die übrigen sind auf Schulhofkarten Kandidaten 545. | `beleuchtung` |
| Register (Berlin) | register | Gepflegtes Verzeichnis einer Objektart: Beleuchtungskataster, Zierbrunnen (`bez_zierbrunnen`, 311) und Trinkwasserbrunnen (313). Ein Registerpunkt verdrängt OSM- und vermessene Punkte derselben Art im Umkreis von 2 m (Laternen) bzw. 3 m (Brunnen) und nennt sie in `freemapper:also`. | `drop_points_near_registers`, `POINT_DEDUPS` |
| Gebäudehöhen (Umweltatlas) | building heights | Ebene `ua_gebaeudehoehen:gebaeudehoehen` mit Daten von 2022; je ALKIS-Gebäude (`gml_id` = uuid) Höhe, Geschosse und Grundrissaktualität. | `gebaeudehoehen` |
| Grundrissaktualität | footprint date | Stand des Gebäudegrundrisses laut Gebäudehöhen 2022 (`grundrissaktualitaet`). Wird für Berliner Gebäude zum Datenstand `freemapper:currency`; kein Vermessungsdatum. Ohne Treffer bleibt das Gebäude ohne Datum. | `join_footprint_dates`, `aktualit` |
| Grünstreifen (Straßenbefahrung) | green strip | Begrünte Fläche im Straßenraum laut Straßenbefahrung (`ce_gruenflaeche`). Wird aus den 501.5-Kartenobjekten der ALKIS-Straßenflächen ausgeschnitten, wenn die Überlappung mindestens 1 mm auf Papier breit ist (höchstens aber 2 m im Gelände verlangt), und ist mit derselben Grenze Kandidat 401. | `GREEN_TYPE`, `subtract_green` |
| Grünausschnitt | green cut-out | Das Ausschneiden der Grünstreifen aus den befestigten Flächen. Das beschnittene Objekt behält die ALKIS-Herkunft und jedes Reststück trägt für die angrenzenden Grünstreifen `freemapper:cut`, `freemapper:cut_ids` und `freemapper:cut_currency` (ältestes nicht geschätztes Aufnahmedatum, sonst das geschätzte mit `freemapper:cut_currency_guessed=yes`). Reststücke ab 2 m Breite bleiben. | `GreenCut` |
| Baumscheibe | tree pit | Unversiegelte Fläche um einen Straßenbaum (`bn_baumscheibe`, 2–5 m²). Wird nicht ausgeschnitten: unter der ISSprOM-Mindestgröße. | `GREEN_TYPE` |
| Vermessene Befestigung | surveyed paving | Befestigte Flächen der Straßenbefahrung (Fahrbahn, Gehweg, Radweg, Parkfläche, Platz, Fußgängerzone, Gehwegüberfahrt). Der Teil außerhalb der gezeichneten ALKIS-Verkehrsflächen ist Kandidat 501.5 (Sprintkarte). | `SURFACE_TYPES`, `surveyed_paved_candidates` |
| Bordstein (Straßenbefahrung) | surveyed curb | Bordsteinlinie der Straßenbefahrung (`bd_bordstein`): Kandidat 501.1 auf Sprintkarten, nicht auf Schulhofkarten. Die ALKIS-`Fahrbahnbegrenzungslinie` bleibt ungenutzt. | `bd_bordstein` |
| Geländer (Straßenbefahrung) | surveyed railing | Geländer, Verkehrsschutzgitter und Leitplanke der Straßenbefahrung: Kandidat 518 (undurchlässig wie ein Zaun); wer sie überqueren kann, stuft sie auf 516 herab. | `bf_gelaender`, `bb_verkehrsschutzgitter`, `bg_leitplanke` |
| Treppenstufenzahl | step count | Das Feld `trs` der vermessenen Treppen (`cc_treppe`), vermutlich die Stufenzahl (keine Codeliste); steht als `berlin:trs` am Kandidaten, `NULL` bleibt weg. | `berlin:trs` |
| Möblierung (Straßenbefahrung) | street furniture | Bänke, Hydranten, Poller, Masten, Fahnenmasten und Spielgeräte der Straßenbefahrung (`sb-moebel`); Kandidaten nur auf Schulhofkarten. | `sb-moebel` |
| Register vor ALKIS vor Straßenbefahrung vor OSM | registers, ALKIS, survey, OSM | Reihenfolge der Quellen bei Doppeln, in jedem Bundesland: Registerpunkte vor allen anderen, dann bei Zäunen, Mauern, Hecken, Bächen, Gräben und Treppen das ALKIS-Objekt (auch das vollständige ALKIS) vor dem vermessenen, dieses vor OSM. Das bleibende Objekt nennt die verworfenen in `freemapper:also` als `<Quelle>/<Quell-ID>` (Bäume ausgenommen). | `Rank`, `drop_points_near_registers`, `drop_cadastre_lines_by_rank`, `same_stair` |
| Rang (Doppel) | duplicate rank | Vertrauen in eine Quelle bei Doppeln, aus `freemapper:source`: Register (Berliner Register, `trees/…`) > ALKIS (`alkis`, `alkis-nas/…`, übrige Geoportal-Berlin-Ebenen) > Straßenbefahrung (`geoportal-berlin/sb-…`) > OSM. Ein Objekt niedrigeren Rangs nahe einem gleichen höheren Rangs fällt weg; bei Flächen (522, 520, Brücken 501.5) entscheidet der bedeckte Anteil, aufgeweitet um 0,25 mm auf dem Papier (ADR 0078). | `Rank`, `rank`, `drop_areas_by_rank`, `build/dedup.rs` |
| Zusammengefügte Linie | joined line | Kurze Linienstücke einer Quelle außer OSM mit demselben Symbol, die Ende an Ende aneinanderstoßen (0,1 m, Knick bis etwa 30°), werden vor der 1-mm-Grenze zu einer Linie; die Tags kommen vom ersten Stück. | `join_touching_lines` |
| Kleingarten | allotment garden | ALKIS-Nutzung „Sport-, Freizeit- und Erholungsfläche / Kleingarten“; privates Gelände, in jedem Bundesland ein Kandidat für 520 (verbotenes Gebiet). | `is_private` |
| K5 | K5 (Karte von Berlin 1:5000) | Die Berlin-Karte 1:5000 in Farbe (WMS `k5_farbe`), als Vorlage `templates/k5.png` mit Weltdatei gespeichert (0,25 m je Pixel, gröber bei Gebieten über 1,5 km Seitenlänge). | `K5_RELPATH`, `K5_WMS` |
| OpenStreetMap (OSM) | OpenStreetMap (OSM) | Freie, gemeinschaftlich erstellte Geodaten. | `Osm` |
| Genehmigungshinweise | permit notes | Dokument neben der Karte (`permits.md`, `permits.json`): welche Verwaltungseinheiten, Schutzgebiete und Betreiber das Kartengebiet berührt und wen man dann meist fragt. Nur auf Wunsch (`freemapper permits`, `run --permits`, MCP `permits_for_area`). Ein Hinweis, keine Rechtsauskunft (ADR 0117). | `permits`, `PermitNotes` |
| Schutzgebiet | protected area | Fläche, die nach Naturschutzrecht geschützt ist: Nationalpark, Nationales Naturmonument, Naturschutzgebiet, FFH-Gebiet, Vogelschutzgebiet, Biosphärenreservat (mit Zonen), Landschaftsschutzgebiet, Naturpark. Die Genehmigungshinweise lesen sie aus dem WFS des BfN, dazu Schutzgebiete aus OSM, die das BfN nicht führt. | `ProtectedArea`, `ProtectedKind` |
| Verwaltungseinheit | administrative unit | Bundesland, Kreis (oder kreisfreie Stadt), Amt oder Verwaltungsgemeinschaft und Gemeinde, aus VG250 des BKG (generalisiert auf 1:250 000). Die Genehmigungshinweise nennen jede mit ihrem Anteil am Kartengebiet. | `AdminUnit`, `AdminLevel` |

## Koordinaten & Georeferenzierung / coordinates & georeferencing

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Koordinatenreferenzsystem (KRS) | coordinate reference system (CRS) | System zur Beschreibung von Positionen, z. B. EPSG:25833. | `Crs` |
| EPSG-Code | EPSG code | Nummer eines KRS in der EPSG-Datenbank. | `epsg` |
| Rechtswert / Hochwert | easting / northing | Projizierte x- bzw. y-Koordinate in Metern. | `easting`, `northing` |
| Georeferenzierung | georeferencing | Verknüpfung von Kartenkoordinaten mit realen Koordinaten. | `Georeferencing` |
| Bezugspunkt | reference point | Punkt, der Karten- und Weltkoordinaten verbindet. | `ref_point` |
| Kartenkoordinate | map coordinate | Lage auf dem Papier, in OOM 1/1000 mm, y nach Süden. | `MapCoord` |
| Missweisung / Deklination | magnetic declination | Winkel zwischen geografisch und magnetisch Nord. | `declination_deg` |
| Meridiankonvergenz | grid convergence | Winkel zwischen Gitter-Nord und geografisch Nord. | `convergence_deg` |
| Abweichung Gitter-Nord zu magnetisch Nord (Fachwort: Grivation) | grivation | Winkel zwischen Gitter-Nord und magnetisch Nord (= Missweisung − Meridiankonvergenz). „Grivation“ ist das Fachwort und der Name des Feldes in OOM- und OCAD-Dateien. Im Text steht die ausgeschriebene Form. | `grivation_deg` |
| Maßstabsfaktor | scale factor | Projektionsbedingte Längenverzerrung (UTM ≈ 0,9996). | `grid_scale_factor` |
| Höhenbezug | vertical datum | Bezugssystem der Höhen (hier DHHN2016). | `VerticalDatum` |

## freemapper-Abläufe / freemapper workflow

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Projekt | project | Ordner mit Projektdatei, Karte, Vorlagen und Namensnennung. | `Project` |
| Projektdatei | project file | `freemapper.toml`: Gebiet, Kartenart, Parameter, Datenstände. | `ProjectFile` |
| Vertrag der Suite | suite contract | Was freemapper in Karte und Projektdatei schreibt und worauf sich freemapper-print und freemapper-courseplan verlassen: Namen der Kartenteile, die Tags `freemapper:source=sheet`, `base` und `north-lines`, die Schlüssel `map_kind`, `title`, `scale_denominator` und `crs`. Festgehalten in `freemapper-contract` (freemapper-core, dort ADR 0018) und in jedem Testlauf geprüft (ADR 0089). | `contract_*` (Tests) |
| Anlegen | init | Projekt mit Kartengebiet und Kartenart erzeugen. | `init` |
| Abrufen | fetch | Benötigte Kacheln laden (über den Zwischenspeicher). | `fetch` |
| Download-Schätzung | download estimate | Was ein Abruf laden wird, je Download mit Größe in MB und ob er schon im Zwischenspeicher liegt. | `DownloadEstimate`, `estimate_download` |
| Speicherplatz | disk space | Freier Platz auf den Datenträgern, auf die freemapper schreibt (Zwischenspeicher, Projektordner); vor jedem Abruf geprüft. | `DiskSpace`, `space::plan` |
| Datenträger | volume | Laufwerk oder Partition. Zwischenspeicher und Projektordner können auf verschiedenen Datenträgern liegen; jeder wird für sich geprüft. | `Volume` |
| Reserve | reserve | Platz, den freemapper auf jedem Datenträger frei lässt (1 GB), damit System und andere Programme weiterlaufen. | `RESERVE_BYTES` |
| Teil (eines Abrufs) | part (of a fetch) | Downloads, die freemapper nur zusammen lädt oder auslässt: Kartendaten (ALKIS, OSM), Geländemodell, Luftbild, Oberflächenmodell, Aktualisierung des OSM-Auszugs. | `Part` |
| Speicherplan | space plan | Welche Teile eines Abrufs auf die Datenträger passen; fehlt Platz, lässt freemapper die entbehrlichen aus und warnt. | `SpacePlan`, `Shortfall` |
| Datenträger voll | disk full | Schreibfehler, weil der Datenträger voll ist (ENOSPC); freemapper löscht die halbe Datei und meldet es klar. | `is_storage_full` |
| Gemeinsame Bibliothek | shared crate | Rust-Crate aus freemapper-core, die mehrere Programme der Suite nutzen; freemapper bindet sie über Git mit festem Release-Tag ein (ADR 0067). | `freemapper_fs` |
| Programmsprache | program language | Sprache der Programmtexte eines Laufs, Deutsch oder Englisch: aus `--lang`, sonst `FREEMAPPER_LANG`, sonst der Sprache des Systems, sonst Englisch (ADR 0092). Nicht die Sprache der Karte: Kartentexte und Quellenangaben bleiben, wie die Karte sie braucht. | `text::lang`, `--lang` |
| Textkatalog | catalog | Die Programmtexte in allen Sprachen als Fluent-Dateien, Englisch als Referenz (`i18n/<Sprache>/freemapper.ftl`). | `text::TEXTS` |
| Ableiten | derive | Hilfsprodukte berechnen (Schummerung, Höhenlinien, nDOM …). | `derive` |
| Erzeugen | build | Kartenobjekte, Kandidaten und Vorlagen erstellen. | `build` |
| Kartenaufbau | map assembly | Die Objekte aller Quellen in fester Schrittfolge zu Karte und Kandidaten zusammensetzen: Doppel auflösen, Punkte generalisieren, nach den Inhaltseinstellungen platzieren. Bekommt die Angaben der Projektdatei (`Frame`) und die gelesenen gespeicherten Daten (`Stored`), liest selbst keine Datei (ADR 0115). | `build::assembly`, `build`, `Frame`, `Stored`, `Built`, `Placed` |
| Exportieren | export | OOM-Datei und Vorlagen in den Projektordner schreiben. | `export` |
| Karten-PDF | map PDF | Druckfertige PDF-Karte mit genau den Objekten der OOM-Karte, ohne Kandidaten und standardmäßig ohne Maßstabsleiste (ADR 0080). | `export_pdf` |
| Trainingskarte | training map | Voll generierte, druckfertige PDF-Karte, in der alle Kandidaten als Kartenobjekte gezeichnet sind (`<name>-candidates.pdf`). | `export_candidates_pdf` |
| Maßstabsleiste | scale bar | Grafischer Maßstab im Titelblock; im PDF nur mit `--pdf-scale-bar`, die Maßstabszahl steht immer da. | `SCALE_BAR_SOURCE` |
| Quellenblock | source block | Die Quellenzeilen unter dem Titelblock des PDFs; `--pdf-sources auto` kürzt sie bei Bedarf, `credit-only` lässt nur die freemapper-Zeile (ADR 0081). | `PdfSources`, `source_block` |
| OCAD-Datei | OCD file | Karte im Binärformat von OCAD (freemapper schreibt Version 12). | `ocd`, `export_ocad` |
| Farbtabelle | colour table | Farben des Symbolsatzes mit Name, Priorität und CMYK; freemapper nutzt die Werte der IOF-Druck- und Farbdefinitionen Rev. 4 (ADR 0044). | `colors_xml`, `RenderColour`, `OcdColour` |
| Volltonfarbe | spot colour | Benannte Druckfarbe (PURPLE, BLACK, GREEN, BLUE, BROWN, YELLOW), aus der OOM die Rasterfarben ableitet; maßgeblich für den Druck ist das CMYK. | `<spotcolors>` |
| Platzhaltersymbol | placeholder symbol | Symbol mit Nummer, Name und Hauptfarbe, aber vereinfachter Grafik; in OCAD durch den offiziellen Satz ersetzt. | `Placeholder` |
| Gebietsauswahl | area selection | Kartengebiet auf einer lokalen Browserseite suchen oder zeichnen und speichern. | `select`, `Selector`, `open_area_selector` |
| Region | region | Bundesland mit eigenen Datenquellen (Brandenburg, Berlin, Sachsen, Sachsen-Anhalt, Hamburg, Hessen, Schleswig-Holstein, Mecklenburg-Vorpommern, Nordrhein-Westfalen, Niedersachsen, Rheinland-Pfalz, Saarland, Baden-Württemberg, Bremen); ein Kartengebiet an der Landesgrenze gehört zu mehreren. Jede Region hat einen Regionsadapter und in der Projektdatei eine Kennung (`brandenburg`). | `AdapterId`, `regions_for` |
| Rasterregion | raster region | Die Region, in der die Mitte des Kartengebiets liegt; aus ihr kommen Luftbild, Geländemodell und nDOM. | `raster_region` |
| Abdeckung | coverage | Die Fläche aller Regionen zusammen. Reicht ein Kartengebiet mehr als 50 m darüber hinaus (ins Ausland, aufs Meer oder in ein Bundesland, das freemapper noch nicht unterstützt, etwa Bayern oder Thüringen), warnt freemapper beim Laden. | `reaches_beyond_coverage`, `BEYOND_COVERAGE_WARNING` |
| Alterswarnung (DOM) | surface age warning | Warnung, wenn das DOM mehr als 24 Monate älter ist als das Luftbild (Niedersachsen) oder aus einem früheren Jahr stammt (Bremen); das nDOM kann dann Neubauten und junge Bäume nicht zeigen. | `lgln::surface_age_warning`, `SURFACE_AGE_WARNING_MONTHS` |
| Anbieter | provider | Stelle, die die Rasterdaten einer Region liefert: LGB (Brandenburg, Berlin), GeoSN (Sachsen), LVermGeo (Sachsen-Anhalt), LGV (Hamburg), HVBG (Hessen), LVermGeo SH (Schleswig-Holstein), LAiV M-V (Mecklenburg-Vorpommern), Geobasis NRW (Nordrhein-Westfalen), LGLN (Niedersachsen), LVermGeo RP (Rheinland-Pfalz), LVGL (Saarland), LGL (Baden-Württemberg) oder das Landesamt GeoInformation Bremen (Bremen). In der Projektdatei steht seine Kennung (`lgb`). | `ProviderInfo`, `raster_provider` |
| Koordinatensystem (Zone) | CRS (zone) | ETRS89/UTM-Zone 32 (EPSG:25832) oder 33 (EPSG:25833); jede Karte bekommt die Zone ihrer Rasterregion. | `Crs`, `ProjectFile::crs` |
| Landesgrenze (Berlin) | Berlin boundary | Umriss Berlins aus ALKIS (`alkis:land`), 25 m vereinfacht; entscheidet über die Region. | `boundaries::BERLIN` |
| Vorschau | preview | PNG-Bild einer Ebene für Mensch oder Agent. | `Preview` |
| Ebene | layer | Ein Ergebnis (Objekte, Kandidaten oder Vorlage) mit einheitlichem Inhalt. | `Layer` |
| Werkzeug (MCP) | tool (MCP) | Vom MCP-Server angebotene Funktion für Agenten. | `#[tool]` |
| Selbstaktualisierung | self-update | freemapper ersetzt sich selbst durch eine neuere Version aus den GitHub-Releases, am selben Pfad, nach Prüfung von Signatur und Prüfsummen; die ersetzte Version bleibt als `freemapper.previous` (ADR 0094). | `freemapper_update`, `update` |
| Aktualisierung (des Programms) | update | Ein Lauf der Selbstaktualisierung: prüfen, laden, prüfen, austauschen. Nicht zu verwechseln mit der Aktualisierung des OSM-Auszugs im Zwischenspeicher. | `update::command`, `install` |
| Aktualisierungsmanifest | update manifest | `freemapper-update.json` in jedem Release: Version, Datum, Versionshinweise, kleinste Version, die sich selbst aktualisieren kann, und je System Adresse, Größe und SHA-256 der Programmdatei; mit minisign signiert (`.minisig`). | `Manifest`, `parse_manifest` |
| Aktualisierungsmodus | update mode | Was die tägliche Prüfung bei einer neueren Version tut: `notify` (Hinweis, Standard), `install` (installiert nach einem gelungenen Befehl) oder `off`; in `settings.toml`, überschreibbar mit `FREEMAPPER_UPDATE`. | `Mode`, `update --mode` |
| Tägliche Prüfung | daily check | Prüfung auf eine neuere Version, höchstens einmal in 24 Stunden, nach `init`, `select`, `fetch` und `run` und im MCP-Server; der letzte Versuch steht in `update-check.json` im Zwischenspeicher. | `start_background_check`, `CheckRecord` |
