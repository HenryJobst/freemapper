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
| Gebäude | building | Gebäudeumriss (ISSprOM 521). | `Building` |
| Überdachung | canopy | Überdachte, begehbare Fläche, z. B. Arkade (ISSprOM 522). Nicht Baumkrone! In Berlin werden die ALKIS-Bauwerke `Überdachung` und `Carport` zu Kartenobjekten. | `Canopy`, `canopy_objects` |
| Treppe | stairway | Treppe als Linie in Laufrichtung (ISSprOM 532.6–532.9); in Berlin Kandidaten aus den ALKIS-Bauwerken `Treppe` und `Freitreppe`. | `stair_objects` |
| Laufrichtung (Treppe) | walking direction | Richtung, in der man eine Treppe hinauf- oder hinabgeht. freemapper nimmt die Achse des Treppenumrisses mit dem größeren Höhenunterschied im DGM; steigt keine um 0,3 m, die lange Achse, markiert mit `freemapper:direction=guessed`. | `STAIR_MIN_RISE_M` |
| Treppenbreite | stair footprint | Breite der Treppe quer zur Laufrichtung in Metern; wählt das Symbol 532.6 (1,6 m) bis 532.9 (4 m). | `footprint_code` |
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
| Verbotenes Gebiet / Privatgelände | out-of-bounds area | Nicht zu betretende Fläche, olivgrün (ISSprOM 520); Kandidaten aus der ALKIS-Nutzung. | `out_of_bounds_objects` |
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
| Randangaben | peripheral information | Maßstab, Äquidistanz, Titel, Maßstabsleiste, Copyright (später Nordbalken, Legende) außerhalb des Kartenbilds, im Kartenteil „peripheral information“. | `PeripheralInfo` |
| Nordlinien | magnetic north lines | Linien nach magnetisch Nord über dem Kartenbild (ISSprOM 601, Abstand 30 mm × Symbolvergrößerung). Kartenobjekte im Kartenteil „default part“, nicht Teil der Randangaben (ADR 0043). | `north_lines` |
| Maßstabsleiste | bar scale | Strecke mit Teilstrichen und Metern im Gelände, Teil der Randangaben. | `bar_scale` |
| Baumkrone | tree crown | Vom Laserscan erfasstes Blätterdach eines Baumes. | `TreeCrown` |

## Geodaten / geodata

| Deutsch | English | Definition | Code |
|---|---|---|---|
| Geobasisdaten | official geodata | Amtliche Vermessungsdaten der Länder (hier: LGB Brandenburg). | – |
| Datenquelle | data source | Anbieter + Produkt, z. B. LGB DGM1, OSM. | `DataSource` |
| Quelladapter | source adapter | Code, der eine Datenquelle findet, lädt und normiert. | `SourceAdapter` |
| Produkt | product | Einzelner Datensatztyp einer Quelle, z. B. DGM1, nDOM, DOP20. | `Product` |
| Kachel | tile | Rechteckiges Teilstück eines Produkts (LGB: 1×1 km). | `Tile` |
| Kachelliste | tile list | Liste eines Anbieters mit Namen (und Datum) der verfügbaren Kacheln; bei Geobasis NRW die `*_meta.zip` je Produkt, bei LVermGeo RP die WFS-Ebene `Laserbefliegung` (Flugtage je Kachel, daraus das Jahr im Dateinamen). | `meta_list`, `lvermgeo_rp::tile_list` |
| Befliegungsübersicht (Laserbefliegung) | laser-scan flight index | WFS-Ebene `Laserbefliegung` des LVermGeo RP: je 1-km-Kachel Flugjahr und Flugtage des Laserscans. freemapper bildet daraus den Dateinamen der DGM1- und DOM1-Kachel und nimmt den letzten Flugtag als Datum. | `lvermgeo_rp::TILE_LIST_URL`, `ListedTile` |
| Metalink | Metalink | XML-Datei (`.meta4`), die Namen, Größen und Prüfsummen der Dateien eines Servers nennt. LVermGeo RP veröffentlicht je Produkt eine für das ganze Land (etwa 11 MB); freemapper liest sie nur für Kacheln, denen die Befliegungsübersicht kein Flugjahr gibt. | `lvermgeo_rp::metalink_url`, `parse_metalink` |
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
| Regierungsbezirk-Auszug | government district extract | OSM-Auszug eines Regierungsbezirks (Nordrhein-Westfalen: Arnsberg, Detmold, Düsseldorf, Köln, Münster) statt des ganzen Landes; gewählt nach der mitgelieferten Auszugsgrenze, an einer Bezirksgrenze mehrere. | `GovernmentDistrict`, `extracts_for` |
| Weltdatei | world file | Textdatei mit Pixelgröße und Lage eines Rasterbilds (`.jgw`, `.pgw`, `.tfw`). | `GridSpec::from_world_file` |
| Mosaik | mosaic | Zusammengesetzte Kacheln, auf Kartengebiet plus Puffer zugeschnitten. | `mosaic_dtm` |
| Geländemosaik | terrain mosaic | Das DGM-Mosaik eines Kachelanbieters, das `derive` für Gebiete mit Berlin-Zusatzebenen unter `data/terrain.tif` speichert, damit der Export die Laufrichtung der Treppen offline bestimmen kann. | `TERRAIN_MOSAIC_RELPATH` |
| Rasterzelle | cell | Ein Pixel eines georeferenzierten Rasters. | `Cell` |
| Auflösung | resolution | Kantenlänge einer Rasterzelle in Metern. | `resolution_m` |
| Liegenschaftskataster (ALKIS) | cadastre (ALKIS) | Amtliches Kataster: Flurstücke, Gebäude, Nutzung. | `Alkis` |
| Flurstück | parcel | Katasterfläche mit Eigentumsgrenze. | `Parcel` |
| Tatsächliche Nutzung | actual land use | ALKIS-Klassifikation der Flächennutzung (`nutzart`). | `LandUse` |
| Berlin-Zusatzebene | extra Berlin layer | Weitere WFS-Ebene des Geoportals Berlin (ALKIS-Bauwerke, Relief, Hecken, Baumbestand), unter `data/berlin/<Name>.geojson` gespeichert und nur für Gebiete bei Berlin geladen. | `ExtraLayer`, `EXTRA_LAYERS` |
| Bauart (Bauteil) | part kind (building part) | Art eines ALKIS-Bauteils, z. B. „Durchfahrt im Gebäude“, „Arkade“ (`bezbat`). | `bauart` |
| Berlin-Regeln | Berlin rules | Regeldatei `berlin-candidates.toml`: macht aus Geoportal-Berlin-Objekten (Tags `berlin:<Eigenschaft>`) Kandidaten, über dieselbe Regelmaschine wie die OSM-Regeln. | `Rules::berlin`, `berlin_objects` |
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
| Download-Schätzung | download estimate | Was ein Abruf laden wird, je Download mit Größe in MB und ob er schon im Zwischenspeicher liegt. | `DownloadEstimate`, `estimate_download` |
| Speicherplatz | disk space | Freier Platz auf den Datenträgern, auf die freemapper schreibt (Zwischenspeicher, Projektordner); vor jedem Abruf geprüft. | `DiskSpace`, `space::plan` |
| Datenträger | volume | Laufwerk oder Partition. Zwischenspeicher und Projektordner können auf verschiedenen Datenträgern liegen; jeder wird für sich geprüft. | `Volume` |
| Reserve | reserve | Platz, den freemapper auf jedem Datenträger frei lässt (1 GB), damit System und andere Programme weiterlaufen. | `RESERVE_BYTES` |
| Teil (eines Abrufs) | part (of a fetch) | Downloads, die freemapper nur zusammen lädt oder auslässt: Kartendaten (ALKIS, OSM), Geländemodell, Luftbild, Oberflächenmodell, Aktualisierung des OSM-Auszugs. | `Part` |
| Speicherplan | space plan | Welche Teile eines Abrufs auf die Datenträger passen; fehlt Platz, lässt freemapper die entbehrlichen aus und warnt. | `SpacePlan`, `Shortfall` |
| Datenträger voll | disk full | Schreibfehler, weil der Datenträger voll ist (ENOSPC); freemapper löscht die halbe Datei und meldet es klar. | `is_storage_full` |
| Ableiten | derive | Hilfsprodukte berechnen (Schummerung, Höhenlinien, nDOM …). | `derive` |
| Erzeugen | build | Kartenobjekte, Kandidaten und Vorlagen erstellen. | `build` |
| Exportieren | export | OOM-Datei und Vorlagen in den Projektordner schreiben. | `export` |
| Trainingskarte | training map | Voll generierte, druckfertige PDF-Karte, in der alle Kandidaten als Kartenobjekte gezeichnet sind. | `export_training_pdf` |
| OCAD-Datei | OCD file | Karte im Binärformat von OCAD (freemapper schreibt Version 12). | `ocd`, `export_ocad` |
| Farbtabelle | colour table | Farben des Symbolsatzes mit Name, Priorität und CMYK; freemapper nutzt die Werte der IOF-Druck- und Farbdefinitionen Rev. 4 (ADR 0044). | `colors_xml`, `RenderColour`, `OcdColour` |
| Volltonfarbe | spot colour | Benannte Druckfarbe (PURPLE, BLACK, GREEN, BLUE, BROWN, YELLOW), aus der OOM die Rasterfarben ableitet; maßgeblich für den Druck ist das CMYK. | `<spotcolors>` |
| Platzhaltersymbol | placeholder symbol | Symbol mit Nummer, Name und Hauptfarbe, aber vereinfachter Grafik; in OCAD durch den offiziellen Satz ersetzt. | `Placeholder` |
| Gebietsauswahl | area selection | Kartengebiet auf einer lokalen Browserseite suchen oder zeichnen und speichern. | `select`, `Selector`, `open_area_selector` |
| Region | region | Land mit eigenen Datenquellen (Brandenburg, Berlin, Sachsen, Sachsen-Anhalt, Hamburg, Hessen, Schleswig-Holstein, Mecklenburg-Vorpommern, Nordrhein-Westfalen, Niedersachsen, Rheinland-Pfalz); ein Kartengebiet an der Landesgrenze gehört zu mehreren. | `Region`, `regions_for` |
| Rasterregion | raster region | Die Region, in der die Mitte des Kartengebiets liegt; aus ihr kommen Luftbild, Geländemodell und nDOM. | `raster_region` |
| Abdeckung | coverage | Die Fläche aller Regionen zusammen. Reicht ein Kartengebiet mehr als 50 m darüber hinaus (ins Ausland, aufs Meer oder in ein Land, das freemapper noch nicht unterstützt, etwa Bremen oder Thüringen), warnt freemapper beim Laden. | `reaches_beyond_coverage`, `BEYOND_COVERAGE_WARNING` |
| Alterswarnung (DOM) | surface age warning | Warnung, wenn das DOM mehr als 24 Monate älter ist als das Luftbild (Niedersachsen); das nDOM kann dann Neubauten und junge Bäume nicht zeigen. | `lgln::surface_age_warning`, `SURFACE_AGE_WARNING_MONTHS` |
| Anbieter | provider | Stelle, die die Rasterdaten einer Region liefert: LGB (Brandenburg, Berlin), GeoSN (Sachsen), LVermGeo (Sachsen-Anhalt), LGV (Hamburg), HVBG (Hessen), LVermGeo SH (Schleswig-Holstein), LAiV M-V (Mecklenburg-Vorpommern), Geobasis NRW (Nordrhein-Westfalen), LGLN (Niedersachsen) oder LVermGeo RP (Rheinland-Pfalz). | `Provider` |
| Koordinatensystem (Zone) | CRS (zone) | ETRS89/UTM-Zone 32 (EPSG:25832) oder 33 (EPSG:25833); jede Karte bekommt die Zone ihrer Rasterregion. | `Crs`, `ProjectFile::crs` |
| Landesgrenze (Berlin) | Berlin boundary | Umriss Berlins aus ALKIS (`alkis:land`), 25 m vereinfacht; entscheidet über die Region. | `BERLIN_BOUNDARY` |
| Vorschau | preview | PNG-Bild einer Ebene für Mensch oder Agent. | `Preview` |
| Ebene | layer | Ein Ergebnis (Objekte, Kandidaten oder Vorlage) mit einheitlichem Inhalt. | `Layer` |
| Werkzeug (MCP) | tool (MCP) | Vom MCP-Server angebotene Funktion für Agenten. | `#[tool]` |
