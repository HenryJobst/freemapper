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
| Symbolzuordnung | symbol mapping | Regel, welche Quelldaten zu welchem Symbol werden. | `SymbolMapping` |
| Kartenobjekt | map object | Einzelnes gezeichnetes Element mit Symbol und Geometrie. | `MapObject` |
| Kartenteil | map part | Gruppe von Kartenobjekten in OOM (Ebene). | `MapPart` |
| Kartenvorlage | template | Hintergrunddatei (Raster oder Vektor) in OOM, die nicht Teil der Karte ist und zum Abzeichnen dient. | `Template` |
| Vorlagenkarte | template map | OOM-Datei, die in einer anderen OOM-Datei als Kartenvorlage eingebunden ist. | `TemplateMap` |
| Kandidat | candidate | Unsicher abgeleitetes Objekt, das nur als Vorlage geliefert wird (z. B. Privatgelände, Barriere). | `Candidate` |
| Startprojekt | starter project | Erstes, von freemapper erzeugtes OOM-Projekt, ab dem in OOM kartiert wird. | `StarterProject` |
| Abgleich | reconciliation | (später) Vergleich neuer Quelldaten mit einer bereits bearbeiteten Karte. | `reconcile` |
| Generalisierung | generalization | Vereinfachen von Geometrie auf Kartenmaßstab (Mindestgrößen, Vereinfachung). | `generalize` |
| Glättung | smoothing | Entfernen von Rauschen aus Linien oder Rastern. | `smooth` |
| Namensnennung | attribution | Pflichtvermerk der Datenlizenz auf Karte und in Dateien. | `Attribution` |

## Kartenzeichen (Auswahl ISSprOM 2019) / map features (ISSprOM 2019 selection)

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Höhenlinie | contour | Linie gleicher Höhe (ISSprOM 101). | `Contour` |
| Zählkurve | index contour | Hervorgehobene Höhenlinie, jede fünfte (ISSprOM 102). | `IndexContour` |
| Formlinie | form line | Zwischenhöhenlinie für Geländeformen (ISSprOM 103). | `FormLine` |
| Äquidistanz | contour interval | Höhenabstand benachbarter Höhenlinien in Metern (Sprint: 2 m). | `contour_interval_m` |
| Gebäude | building | Gebäudeumriss (ISSprOM 521). | `Building` |
| Überdachung | canopy | Überdachte, begehbare Fläche, z. B. Arkade (ISSprOM 522). Nicht Baumkrone! | `Canopy` |
| Befestigte Fläche | paved area | Asphalt-, Pflaster- oder Plattenfläche (ISSprOM 501). | `PavedArea` |
| Unbefestigter Weg | unpaved path | Weg/Pfad ohne Befestigung (ISSprOM 505/506/507). | `UnpavedPath` |
| Verbotenes Gebiet / Privatgelände | out-of-bounds area | Nicht zu betretende Fläche, olivgrün (ISSprOM 520). | `OutOfBoundsArea` |
| Unpassierbares Objekt / Durchgangsbarriere | impassable feature | Darf nicht überquert werden: Mauer, Zaun, Hecke usw. | `ImpassableFeature` |
| Zaun | fence | Linienhafte Barriere aus Draht/Holz/Metall. | `Fence` |
| Mauer | wall | Linienhafte Barriere aus Stein/Beton. | `Wall` |
| Hecke | hedge | Linienhafte Barriere aus Pflanzen. | `Hedge` |
| Hydrant | hydrant | Hydrant, Handpumpe, markanter Wasserpunkt (Schulsatz 321). | `Hydrant` |
| Spielgerät | playground equipment | Spielgerät oder Handballtor (Schulsatz 541). | `PlaygroundEquipment` |
| Tisch | table | Tischtennis- oder Picknicktisch (Schulsatz 542). | `Table` |
| Bank | bench | Feste Sitzgelegenheit (Schulsatz 543). | `Bench` |
| Mast | pole | Fahnenmast, Schild, Basketballkorb (Schulsatz 544). | `Pole` |
| Laterne | lamppost | Straßenlaterne oder Ampel (Schulsatz 545). | `Lamppost` |
| Tartanbahn | tartan track | Kunststoffbelag von Laufbahn/Sportfeld, im Schulsatz 501 mit 50 % Braun. | `TartanTrack` |
| Randangaben | peripheral information | Maßstab, Äquidistanz, Titel, Maßstabsleiste, Copyright (später Nordbalken, Legende) außerhalb des Kartenbilds. | `PeripheralInfo` |
| Nordlinien | magnetic north lines | Linien nach magnetisch Nord über dem Kartenbild (ISSprOM 601, Abstand 30 mm × Symbolvergrößerung). | `north_lines` |
| Maßstabsleiste | bar scale | Strecke mit Teilstrichen und Metern im Gelände, Teil der Randangaben. | `info_block` |
| Baumkrone | tree crown | Vom Laserscan erfasstes Blätterdach eines Baumes. | `TreeCrown` |

## Geodaten / geodata

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Geobasisdaten | official geodata | Amtliche Vermessungsdaten der Länder (hier: LGB Brandenburg). | – |
| Datenquelle | data source | Anbieter + Produkt, z. B. LGB DGM1, OSM. | `DataSource` |
| Quelladapter | source adapter | Code, der eine Datenquelle findet, lädt und normiert. | `SourceAdapter` |
| Produkt | product | Einzelner Datensatztyp einer Quelle, z. B. DGM1, nDOM, DOP20. | `Product` |
| Kachel | tile | Rechteckiges Teilstück eines Produkts (LGB: 1×1 km). | `Tile` |
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
| Punktwolke | point cloud | Menge von 3D-Messpunkten (LAS/LAZ). | `PointCloud` |
| Laserscan / ALS | airborne laser scanning (ALS) | Flugzeuggestützte LiDAR-Vermessung. | `Als` |
| Bodenpunkt | ground point | Punkt der Klasse „Boden“ in einer Punktwolke. | `GroundPoint` |
| Digitales Orthophoto (DOP) | orthophoto | Entzerrtes, georeferenziertes Luftbild (DOP20 = 0,2 m). | `Orthophoto` |
| Weltdatei | world file | Textdatei mit Pixelgröße und Lage eines Rasterbilds (`.jgw`, `.pgw`, `.tfw`). | `GridSpec::from_world_file` |
| Mosaik | mosaic | Zusammengesetzte Kacheln, auf Kartengebiet plus Puffer zugeschnitten. | `mosaic_dtm` |
| Rasterzelle | cell | Ein Pixel eines georeferenzierten Rasters. | `Cell` |
| Auflösung | resolution | Kantenlänge einer Rasterzelle in Metern. | `resolution_m` |
| Liegenschaftskataster (ALKIS) | cadastre (ALKIS) | Amtliches Kataster: Flurstücke, Gebäude, Nutzung. | `Alkis` |
| Flurstück | parcel | Katasterfläche mit Eigentumsgrenze. | `Parcel` |
| Tatsächliche Nutzung | actual land use | ALKIS-Klassifikation der Flächennutzung (`nutzart`). | `LandUse` |
| OpenStreetMap (OSM) | OpenStreetMap (OSM) | Freie, gemeinschaftlich erstellte Geodaten. | `Osm` |

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
| Grivation | grivation | Winkel zwischen Gitter-Nord und magnetisch Nord (= Missweisung − Meridiankonvergenz). | `grivation_deg` |
| Maßstabsfaktor | scale factor | Projektionsbedingte Längenverzerrung (UTM ≈ 0,9996). | `grid_scale_factor` |
| Höhenbezug | vertical datum | Bezugssystem der Höhen (hier DHHN2016). | `VerticalDatum` |

## freemapper-Abläufe / freemapper workflow

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Projekt | project | Ordner mit Projektdatei, Karte, Vorlagen und Namensnennung. | `Project` |
| Projektdatei | project file | `freemapper.toml`: Gebiet, Kartenart, Parameter, Datenstände. | `ProjectFile` |
| Anlegen | init | Projekt mit Kartengebiet und Kartenart erzeugen. | `init` |
| Abrufen | fetch | Benötigte Kacheln laden (über den Zwischenspeicher). | `fetch` |
| Ableiten | derive | Hilfsprodukte berechnen (Schummerung, Höhenlinien, nDOM …). | `derive` |
| Erzeugen | build | Kartenobjekte, Kandidaten und Vorlagen erstellen. | `build` |
| Exportieren | export | OOM-Datei und Vorlagen in den Projektordner schreiben. | `export` |
| Gebietsauswahl | area selection | Kartengebiet auf einer Browserkarte zeichnen. | `select` |
| Vorschau | preview | PNG-Bild einer Ebene für Mensch oder Agent. | `Preview` |
| Ebene | layer | Ein Ergebnis (Objekte, Kandidaten oder Vorlage) mit einheitlichem Inhalt. | `Layer` |
| Werkzeug (MCP) | tool (MCP) | Vom MCP-Server angebotene Funktion für Agenten. | `#[tool]` |
