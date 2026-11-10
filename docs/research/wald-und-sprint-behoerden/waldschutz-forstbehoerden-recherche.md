# Recherche: Schutzstatus, Forst- und Genehmigungsbehörden, Eigentümer je Kartengebiet

Stand der Recherche: 2026-11-10. Nur lesende Abfragen (GetCapabilities, einzelne Mini-Requests). Keine personenbezogenen Daten in Requests.
Alle Angaben sind ein Hinweis für die Planung, keine Rechtsauskunft.

Legende "verifiziert": **ja** = GetCapabilities oder ein Mini-GetFeature wurde in dieser Sitzung selbst abgerufen und gelesen. **teilweise** = Dienst existiert laut Katalog/Suchtreffer, Details nicht selbst abgerufen. **nein** = nur Hinweis aus Suche/Metadaten.
Rohdaten der Capabilities liegen in `.../scratchpad/caps/`.

---

# Teil A: Wald (Waldkarten, ISOM 2017-2)

## A1. Bundesweit

| Quelle | URL / Layer | CRS | Lizenz / Quellenvermerk | verifiziert |
|---|---|---|---|---|
| BfN Schutzgebiete WFS 2.0.0 | `https://geodienste.bfn.de/ogc/wfs/schutzgebiet?SERVICE=WFS&REQUEST=GetCapabilities&VERSION=2.0.0`. FeatureTypes (Namespace `schutzgebiet:`): `Naturschutzgebiete`, `Landschaftsschutzgebiete`, `Fauna_Flora_Habitat_Gebiete`, `Vogelschutzgebiete`, `Nationalparke`, `Naturparke`, `Biosphaerenreservate`, `Biosphaerenreservate_Zonierung`, `Nationale_Naturmonumente` | Default `urn:ogc:def:crs:EPSG::25832` (keine OtherCRS, Bbox-Abfragen daher in 25832 oder 4326 testen). Ausgabe: GML32, GEOJSON, SHAPE+ZIP, CSV, Geopackage | Capabilities nennen "Nutzungsbestimmungen für die Bereitstellung von Geodaten des Bundes" (GeoNutzV), `AccessConstraints: Nicht für Planungszwecke geeignet.` Kein explizites dl-de-Feld. Anbieter "Bundesamt für Naturschutz (BfN)". Quellenvermerk: Vorschlag "Bundesamt für Naturschutz (BfN), Schutzgebiete in Deutschland, Stand je Layer" (BfN-eigener Wortlaut nicht gefunden) | ja (Capabilities). Attribute per CSV-GetFeature gesehen: `name, bfn_id, bl, id, leg_date, leg_link, leg_titel, natda_id, iucn_kat, jahr, flaeche, status`. Ein Bbox-Test (Berlin-Süd) lieferte 0 Treffer, kein Praxistest mit Treffer |
| BfN Schutzgebiete WMS 1.3.0 | `https://geodienste.bfn.de/ogc/wms/schutzgebiet?SERVICE=WMS&REQUEST=GetCapabilities&VERSION=1.3.0`, Layer wie oben, Gruppe `Schutzgebiete` | u. a. 25832 | gleich; Abstract: einmal jährlich aus Länderdaten zusammengeführt, an VG25-Grenzen angepasst. Stand je Layer (aus Abstract): FFH 2023, VSG 2023, NSG 2024, LSG 2024, Nationalparke 2025, Nat. Naturmonumente 2025, Biosphärenreservate 2025, Naturparke 2025 | ja |
| BfN international | `https://geodienste.bfn.de/ogc/wfs/schutzgebiet_int` (Ramsar, HELCOM, OSPAR, MAB) | | wie BfN | nein (nur Suchtreffer) |
| BfN sonstige | `.../ogc/wfs/INSPIRE_AM_DE_Integratives_Monitoring_Grossschutzgebiete`, `.../landschaften` | | | nein |
| BKG VG250 WFS 2.0.0 | `https://sgx.geodatenzentrum.de/wfs_vg250`; Layer `vg250:vg250_gem` (11 103 Gemeinden, per resultType=hits), `vg250_krs`, `vg250_rbz`, `vg250_lan` | 25832, 25833, 3035 u. a. | dl-de/by-2-0; Quellenvermerk "© BKG (Jahr) dl-de/by-2-0, Datenquellen: ..." plus Veränderungshinweis bei Bearbeitung; auf Webseiten "BKG" und "dl-de/by-2-0" verlinken | ja |
| WDPA / Protected Planet | weltweit, Download mit Registrierung/Bedingungen, für DE nur Dublette zu BfN | | nicht-kommerziell, restriktiv | nein, nicht empfohlen |
| EEA Natura 2000 | EU-Download, Dublette zu BfN FFH/VSG | | EEA, frei | nein, nicht nötig |
| OSM | `boundary=protected_area` mit `protect_class` (1a/1b=Strenges Reservat/Nationalpark-Kernzone, 2=Nationalpark, 4=NSG/Habitat, 5=LSG/Naturpark, 6=Ressourcenreservat), `leisure=nature_reserve`, `boundary=national_park`, `landuse=forest`/`natural=wood` mit `operator`, `owner`, `ownership` (selten), `name`. Wasserschutz: `landuse=...`/`boundary=protected_area` + `protection_title`, in DE lückenhaft. Naturwaldreservate: Community-Thread zum Erfassen (`boundary=protected_area` + `protection_title=Naturwaldreservat`), lückenhaft | WGS84 | ODbL, "© OpenStreetMap contributors"; freemapper nutzt Geofabrik-Extrakte bereits, Tags kommen ohne neuen Download (nur Filter erweitern) | Tag-Schema aus Wissen, nicht live geprüft; Qualität unbekannt |

Zu beachten: BfN-Daten sind jährlich zusammengeführt und "nicht für Planungszwecke geeignet" (also Hinweisdaten). Länder sind aktueller. Bannwald/Schonwald (BW, nicht in unseren elf Ländern) und Naturwaldreservate/Naturwälder liegen bei den Landesforsten, nicht beim BfN.

## A2. Je Bundesland

Hinweis zur Spalte "Schutzgebiete": BfN deckt NSG/LSG/FFH/VSG/NP/Naturpark/BR bundesweit ab. Landesdienste sind nur Aktualisierungs-Upgrade, plus Wasserschutzgebiete und Forstspezifika (Naturwald, Totalreservat, Waldfunktionen), die BfN nicht hat.

| Land | Schutzgebiete-Dienst (zusätzlich zu BfN) | Forstverwaltungsgrenzen | Waldbesitz | Lizenz | verifiziert |
|---|---|---|---|---|---|
| **Brandenburg** | Waldfunktionen-WMS enthält Schutzlayer: `lfu_trv` Totalreservat, `wfk_nwr` Naturwald, `lfu_bbk_*` geschützte Biotope, `lfu_ffhmp_*`, `lfu_ueb100`, `lkrs_nd_pu` Naturdenkmal, `wfk_ewf/ewr` Erholungswald, `wfk_nbt` nicht betretbare Fläche, `wfk_gwg` geschützte Waldgebiete (37 Layer). Landes-Naturschutz-WFS (LfU): nicht geprüft | **Forstreviere** (INSPIRE, Revier-Ebene unter Forstamt): WFS `https://www.brandenburg-forst.de/geoserver/IFUEK/ows?service=WFS&request=GetCapabilities`, FeatureType `IFUEK:AM.ForestManagementArea`, EPSG:25833. WMS gleiche Basis (`.../IFUEK/ows?service=WMS...`). Forstamt/Oberförsterei-Ebene: WMS "Forststrukturen des Landesbetriebes Forst Brandenburg" im Geoportal BB (URL nicht abgerufen) | Kein Eigentumsarten-Layer gefunden. Forstgrundkarte-WMS `https://www.brandenburg-forst.de/ogc/fgk` (Grenzen/Wege, keine Besitzart bestätigt). `fgk_fl_wld` Waldbedeckung (WMS `IFGK`) | Reviere: dl-de/by-2-0, "Quellenvermerk: Landesbetrieb Forst Brandenburg" (Metaver). Waldfunktionen: "Nutzung erfolgt derzeit kostenfrei unter Beachtung des Urheberrechts" | Reviere-WFS ja; Waldfunktionen-WMS und INWFK-WFS (`INWFK:LU.ExistingLandUse`) ja; Forststrukturen-WMS nein; Waldbesitz nein |
| **Berlin** | Schutzgebiete WFS `https://gdi.berlin.de/services/wfs/schutzgebiete`: `h_nsg`, `i_lsg`, `c_ffh`, `b_spa`, `d_np` (Naturpark), `e_fnd`, `a_nd`, `g_glb`, `f_fspr` | **Forstämter** WFS `https://gdi.berlin.de/services/wfs/forstverwalt`, FeatureType `forstverwalt:b_forstverwalt` (Forstamt-Ebene; laut Metadaten auch Reviere/Distrikte, nur Forstamt-Layer in den Capabilities gesehen), EPSG:25833 (Capabilities bieten auch 25832/4326). 4 Forstämter (Grunewald, Köpenick, Pankow, Tegel), 28 Revierförstereien | Berliner Wald überwiegend Landeswald; `liegenschaftsplan` (Attribut `sch` = "Land Berlin"/Bund/anderes Land) | dl-de/zero-2-0 (kein Quellenvermerk nötig, "Geoportal Berlin" höflich). Letzte Revision Forstverwaltung 2020-02-07 | ja (alle Capabilities). FIS-Broker laut Suchtreffer abgeschaltet, `gdi.berlin.de/services/...` ist der Weg |
| **Sachsen** | Dienst nicht gefunden (LfULG/iDA nicht geprüft). WMS `naturwaldzellen` (§29 SächsWaldG) | WMS "Wald nach SächsWaldG" `https://www.forsten.sachsen.de/kartendienste/wald/MapServer/WMSServer`, Layer `Forstbezirke`, `Reviere`, `Abteilungen`, `Waldflaeche_nicht_amtlich`; CRS u. a. 25832/25833 | **Nur PDF**: Forstbezirksübersichtskarte Reviere/Kreise 2025 (Stand Waldeigentumsarten 27.01.2025). Kein Dienst | Capabilities: "Datenlizenz Deutschland 2.0 - Variante mit Namensnennung"; Urheber Staatsbetrieb Sachsenforst | WMS ja; Naturwaldzellen-/Waldfunktionen-WMS nur laut Sachsenforst-Seite (`.../kartendienste/naturwaldzellen/...`, `.../waldfunktionen/...`); keine WFS gefunden |
| **Sachsen-Anhalt** | nicht geprüft | LZW INSPIRE-WMS `https://geodatenportal.sachsen-anhalt.de/ows_INSPIRE_LZW_US?REQUEST=GetCapabilities&SERVICE=WMS`, Layer `US.SpecializedAdministrationOffice` = **Forstämter** (nur Ämter, keine Reviere), plus Rettungspunkte | nicht gefunden | "© LZW LSA, 2015", keine Zugriffsbeschränkung; Lizenz nicht explizit (LVermGeo-Basisdaten dl-de/by-2-0) | WMS ja; Rest nein |
| **Hamburg** | `https://geodienste.hamburg.de/HH_WFS_Schutzgebiete` (NSG/LSG u. a.), `.../HH_WFS_Wasserschutzgebiete` | Kein Forstamt-Layer gefunden. Revierförstereien nur textlich | `HH_WFS_Wald_in_Hamburg` (Holzbodenfläche der Revierförstereien, Stand Forsteinrichtung 2008), `HH_WFS_Waldfunktionen` (Auswahl: Erholungswald, Klimaschutzwald) | dl-de/by-2-0, "Freie und Hansestadt Hamburg, Behörde für Umwelt, Klima, Energie und Agrarwirtschaft (BUKEA)" (WSG: "Behörde für Umwelt und Energie"); CRS 25832/25833/4326 | Capabilities ja; Feature-Typnamen und Attribute nicht ausgelesen |
| **Hessen** | Geoportal Hessen hat Schutzgebiete-WFS (z. B. Nationalpark; NATUREG), nicht einzeln geprüft. HessenForst: Naturwaldentwicklungsflächen als Shapefile | Forstamt/Revier-Layer **nicht gefunden** (nur Rettungspunkte-WMS) | nicht gefunden; ALKIS vereinfacht WFS vorhanden | HessenForst-Nutzungsbedingung laut Katalog: frei, Quelle "Landesbetrieb HessenForst", kein Weiterverkauf; Verwaltungsgrenzen dl-de/zero-2-0 | nein/teilweise |
| **Schleswig-Holstein** | Umweltportal-WFS "Fachthemen Umwelt" vorhanden; Naturwald-SHLF WFS (Umweltportal) mit Revier-Code und Revierförsterei-Name | Datensatz "Reviergrenzen (SHLF)" auf opendata.schleswig-holstein.de (Änderung 2020, Metadaten ohne Lizenz); Forstämter nur als PDF-Übersichtskarte auf forst-sh.de | nicht gefunden | Lizenz Reviergrenzen unklar | nein/teilweise |
| **Mecklenburg-Vorpommern** | Umwelt/Naturschutz-WMS/WFS im GeoPortal.MV (nicht geprüft) | **Forststruktur Landesforst MV**: WMS `https://www.geodaten-mv.de/dienste/forst_struktur_wms`, Layer `reviere` (Forstreviere), `aemter` (Forstämter); EPSG 25832/25833/2398. **WFS** `.../forst_struktur_wfs` (`lfoamv:reviere`, `aemter`, `reviersitz`, `amtssitze`) ist **passwortgeschützt, nur auf Anfrage** (Art. 13(1)(h) INSPIRE) | Forstgrundkarte WMS `.../forst_grundkarte_wms` (Layer `FGK`) öffentlich, WFS gesperrt; Besitzart nicht bestätigt | WMS: "Nutzungsbedingungen: keine", Zugriffsbeschränkung "Urheberrecht" (keine offene Lizenz, Weiterverwendung unklar, bei Landesforst MV nachfragen) | WMS ja, WFS gesperrt (ja, aus Capabilities) |
| **Nordrhein-Westfalen** | LINFOS-WMS `https://www.wms.nrw.de/umwelt/linfos` (Schutzgebiete, laut Katalog); "Kartenlayer Naturschutzgebiete NRW" (LANUK, open.nrw). WFS nicht bestätigt (Test-URL `.../schutzgebiete` lieferte 404) | **Forstamtsgrenzen** (14 Regionalforstämter + Nationalparkforstamt + Lehr- und Versuchsforstamt), **Forstbetriebsbezirke** (= Reviere), forstliche Einrichtungen: WMS `https://www.wms.nrw.de/umwelt/waldNRW`, Layer `forstamtsgrenzen_nrw`, `forstbetriebsbezirke_nrw`, `forstliche_einrichtungen_nrw`, `landeseigener_forstbetrieb_staatswald`; Shapefile-Downloads (25832) auf `https://www.opengeodata.nrw.de/produkte/umwelt_klima/wald_forst/wald/` (Forstamtsgrenzen, Forstbetriebsbezirke, Einrichtungen, Staatswald; Zeitstempel 2024 bis 2026-10) | **Staatswald** als Flurstücksdatensatz (13 % der Waldfläche) mit zuständigem Forstamt; Körperschafts-/Privatwald nicht als Layer | Capabilities: Fees "keine", "Es gelten keine Bedingungen"; Katalog: dl-de/by-2-0 (nicht in Capabilities) | WMS und Download-Verzeichnis ja; Lizenz teilweise |
| **Niedersachsen** | NLWKN Open Data (Schutzgebiete) laut Suchtreffer, nicht geprüft | Statewide Forstamts-Layer **nicht gefunden** (26 Forstämter der NLF laut ML; Forstreviere nur Landkreis Diepholz) | Waldfunktionenkarte (NUMIS, nicht INSPIRE-konform), keine Besitzart-Layer | unbekannt | nein |
| **Rheinland-Pfalz** | Geoportal RLP listet "Schutzgebiete" WFS, "Wasser- und Heilquellenschutzgebiete" WFS 2.0.0, Naturschutz-WMS (ODbL laut Suchtreffer) | **WFS Forstamtsgrenzen Landesforsten RLP**: `https://gis.wald.rlp.de/ags/services/INSPIRE/Forstamtsgrenzen_RLP/MapServer/WFSServer`, FeatureType `Forstamtsgrenzen_RLP:Forstamtsgrenzen_Rheinland-Pfalz`. Plus WFS Erholungswald, Erosions-, Immissions-, Klima-, Lärm-, Sicht-, Trassenschutzwald. Reviere nicht gefunden | nicht gefunden | dl-de/by-2-0 laut Geoportalseite; Quellenvermerk nicht angegeben, Kontakt INSPIRE@wald-rlp.de | teilweise (Server lieferte 403 an curl und WebFetch, daher keine Capabilities gesehen) |

**Fazit Forstverwaltung**: Gut offen (WMS/WFS, verifiziert): Brandenburg (Reviere WFS), Berlin (Forstämter WFS), NRW (Forstämter, Betriebsbezirke, Staatswald), MV (WMS), Sachsen (WMS Forstbezirke/Reviere), Sachsen-Anhalt (nur Forstämter), RLP (Forstämter, noch zu verifizieren). Lücken: Hessen, Niedersachsen, Schleswig-Holstein (nur Teilhinweise), Hamburg (kein Forstamt-Layer). **Waldbesitzarten** gibt es als offenen Dienst nur NRW-Staatswald (Flurstücke) und Berlin (Liegenschaftsplan); sonst Gap. Ersatz: ALKIS-Flurstücke ohne Eigentümer (keine Besitzart), OSM `operator`/`owner` (lückenhaft), Landeswald-Gebietsumriss aus den Forstbetriebs-Layern (Staatswald = Gebiet der Landesforst-Reviere, Körperschafts-/Privatwald aber nicht unterscheidbar).

## A3. Wer erteilt Genehmigungen für OL-Veranstaltungen im Wald (nur Orientierung, teils unsicher)

- **Waldeigentümer**: Immer die Zustimmung des Eigentümers/Bewirtschafters. Staatswald: Landesforst (Forstamt/Revier; BB: LFB-Oberförsterei, NRW: Regionalforstamt, NI: NLF-Forstamt, MV/SH/RP/HE/SN/ST: jeweilige Landesforstanstalt). Körperschaftswald: Gemeinde/Stadt. Privatwald: Eigentümer, oft über Forstbetriebsgemeinschaft/Forstamt vermittelt.
- **Forstbehörde**: Das Betretungsrecht (BWaldG §14, Landeswaldgesetze) deckt Erholungsnutzung, organisierte Veranstaltungen sind in mehreren Ländern anzeige- oder genehmigungspflichtig. Zuständig ist die Untere Forstbehörde (je nach Land Kreis, Landesforst-Forstamt oder Landwirtschaftskammer, z. B. NI: LWK). **Paragrafen und Zuständigkeit je Land nicht verifiziert.**
- **Naturschutz**: Untere Naturschutzbehörde (Kreis/kreisfreie Stadt) bei NSG, FFH, VSG, Nationalpark (Nationalparkverwaltung), oft Befreiung/Ausnahme oder Verträglichkeitsprüfung. In NSG meist Wegegebot.
- **Wasser**: Wasserschutzgebiet Zone I/II schließt Veranstaltungen meist aus, Zone III meist unkritisch (Verordnung prüfen, Untere Wasserbehörde).
- **Jagd**: Jagdpächter/Jagdgenossenschaft informieren (Drückjagd-Zeiten). Zuständig Untere Jagdbehörde, Daten offen nicht gefunden.
- **Verbandsvorgaben**: Ein DOSB-/DOLV-Leitfaden mit Zuständigkeiten je Bundesland wurde **nicht gefunden**. Nur Wettkampfbestimmungen (Regeln), keine Genehmigungsleitlinie. Hinweis im Report: Landesverband/Umweltreferat des DOLV fragen.

---

# Teil B: Sprintkarten in Siedlungen

## B1. Behörden und Stellen, aus der Kartenfläche bestimmbar

| Stelle | Ableitung | Offene Datenquelle | Verifiziert |
|---|---|---|---|
| Land / Kreis / Gemeinde (Gemeinde = Ordnungsamt, Sondernutzung) | Polygon schneiden, Flächenanteile | BKG VG250 WFS (siehe A1), alternativ OSM `admin_level` (4 Land, 6 Kreis, 8 Gemeinde, in Stadtstaaten 9 = Ortsteil/Bezirk) | VG250 ja |
| Bezirk (Berlin, Hamburg) | Polygon schneiden | Berlin `https://gdi.berlin.de/services/wfs/alkis_bezirke` (`alkis_bezirke:bezirksgrenzen`, Attribute `namgem`), dl-de/zero-2-0 | ja (Treffer Friedrichshain-Kreuzberg und Tempelhof-Schöneberg am Viktoriapark-Rand) |
| Grünflächenamt / Straßen- und Grünflächenamt (Park, Sondernutzung Grünanlage) | Grünanlage schneiden, Bezirk lesen | Berlin `.../wfs/gruenanlagen` (Grünanlagenbestand inkl. Spielplätze; Attribute `bezirkname`, `namenr`, `objartname`, `widmung`, `katasterfl`, `baujahr`); Hamburg Transparenzportal Grünanlagen (nicht geprüft); andere Städte meist nur OSM `leisure=park` und Stadtportal | Berlin ja |
| Straßenverkehrsbehörde (Sperrung, verkehrsrechtliche Anordnung nach StVO, Sondernutzung nach Straßenrecht) | = Gemeinde/Kreis; in Großstädten teils Landes- oder Bezirksbehörde. Berlin: Aufgabenteilung Bezirk / Verkehrslenkung (nicht verifiziert) | keine Geometrie offen; nur Zuordnung per Gemeinde und Verweis auf Stadtportal | nein |
| Ordnungsamt | Gemeinde | wie oben | nein |
| Polizei | Hinweis (Anzeige/Abstimmung), je Land Zuständigkeit | keine Geometrie | nein |
| Denkmalschutz | Denkmale schneiden | Berlin `.../wfs/denkmale` (Attribute `typ`, `link` zur Denkmaldatenbank), dl-de/zero-2-0. Andere Länder: je Landesdenkmalamt, nicht geprüft | Berlin ja |
| Schlösserverwaltungen (SPSG Sanssouci/Potsdam, Herrenhäuser Gärten Hannover usw.) | Geschlossene Parkflächen, Anbieter über OSM `operator` oder `owner` bzw. Hinweisliste (nur manuell gepflegt) | OSM (lückenhaft). Zur SPSG-Veranstaltungsgenehmigung keine offene Regel gefunden, direkt anfragen | nein |
| ÖPNV/Bahn | Tram/Bahnflächen aus OSM `railway=*`, `operator=*` | OSM | nein |
| Hochschule / Schulträger | OSM `amenity=university/school/college`, `operator` | OSM (freemapper-Schulhofkarten kennen die Flächen bereits) | nein |
| Schutz-/Naturstatus im Siedlungsgebiet | wie Teil A (Parks sind oft LSG, Gartendenkmal, Biotope) | BfN, Berlin `.../wfs/schutzgebiete` | Berlin ja |

Genehmigungspraxis Sprint (Orientierung, aus kommunalen Leitfäden und Landessportbund-Hinweisen, nicht OL-spezifisch): Öffentliche Straßen/Gehwege/Grünanlagen über den Gemeingebrauch hinaus = Sondernutzungserlaubnis (Ordnungs-/Straßenbaulastträger-Amt, bei Grünflächen Grünflächenamt); Berlin: gewerbliche Sportkurse im Park sind erlaubnispflichtig (VG Berlin, Fitnesskurse), Veranstaltungen sind "besondere Nutzungen" der Grünanlagen; Haltverbote/Ausnahmen nach StVO bei der Straßenverkehrsbehörde; Haftpflicht gefordert. Zuständigkeit variiert stark je Stadt (Ordnungsamt, Liegenschaftsamt, SÖR in Nürnberg), daher nur Hinweis und Link, keine Automatik.

## B2. Eigentümer: was offen ist, was nicht

**Rechtlich nicht automatisch**: Eigentümerdaten aus ALKIS/Grundbuch sind nicht offen (Grundbuch: berechtigtes Interesse nach GBO §12; DSGVO bei Privatpersonen). Offene ALKIS-Exporte sind "ohne Eigentümer" (BW, NI, NRW u. a.; Bayern kostenpflichtig). Ein Attribut "Eigentumsart" in offenen ALKIS-Exporten wurde nirgends bestätigt (nicht gefunden, nicht ausgeschlossen).

**Offen auswertbar (verifiziert)**
- **Berlin Liegenschaftsplan** `https://gdi.berlin.de/services/wfs/liegenschaftsplan` (FeatureType `liegenschaftsplan:liegenschaftsplan`): Flurstücke im Eigentum des Landes Berlin, des Bundes oder anderer Länder. Attribute `gisid, fsk, art, sch, bem`; `sch` hatte im Test den Wert "Land Berlin". dl-de/zero-2-0, Aktualisierung jährlich (Metadaten). Die Codeliste für `art` (5920) nicht gefunden. 60 Flurstücke im Viktoriapark-Bbox.
- **Berlin Landesgrundvermögen** `.../wfs/landesgrundvermoegen`: Layer `d_fme_haupt` (Hauptverwaltungen), `e_fme_bezirk` (Bezirksverwaltungen), `f_andere_bereiche`. Zeigt die verwaltende Stelle, nicht Wohnungsbaugesellschaften. dl-de/zero-2-0. Nicht auf Inhalt geprüft.
- **Berlin Grünanlagen**: Bezirk als zuständige Stelle.
- Hinweis aus Suche (nicht geprüft): Berlin veröffentlicht Eigentümerstruktur im Wohnungsmarkt nur auf Kiez-Ebene, nicht grundstücksgenau.

**Teilweise per OSM**: `operator`, `owner`, `name` an Gebäuden und Flächen (z. B. landeseigene Wohnungsbaugesellschaften, Genossenschaften, Kirchen `amenity=place_of_worship` + `operator`, Schulen, Hochschulen, Bahn `operator=DB InfraGO`). Abdeckung nicht systematisch, Qualität unbekannt.

**Nicht offen gefunden**: BImA-Liegenschaften (nur interne GIS-Fallstudien), DB-InfraGO-Flächen, Kirchenland, Bestände der Wohnungsbaugesellschaften als Open-Data-Layer, Hamburg-Liegenschaften (nicht geprüft), Eigentümer der meisten Privatflächen.

**Ehrlicher Report-Text für Unbestimmbares**: "Eigentümer dieser Flächen sind aus offenen Daten nicht bestimmbar. Bei berechtigtem Interesse (z. B. Veranstalter) kann beim Grundbuchamt ein Grundbuchauszug beantragt werden; für öffentliche Flächen hilft das zuständige Ordnungs- oder Grünflächenamt."

## B3. Beispiel Viktoriapark Berlin-Kreuzberg (aus verifizierten offenen Daten; Abfrage mit der Projekt-Bbox 389613/5816190/390529/5816909 in EPSG:25833)

- Bezirke: Friedrichshain-Kreuzberg (Hauptanteil), Tempelhof-Schöneberg (Randstreifen). Quelle `alkis_bezirke`, dl-de/zero-2-0.
- Grünanlagen (`gruenanlagen`, 7 Objekte): Viktoriapark (115 957 m², Baujahr 1894, gewidmet, Bezirk Friedrichshain-Kreuzberg, Ortsteil Kreuzberg), Wolfsschlucht (4 959 m²), Rosengarten (2 688 m²), Tiergehege (1 097 m²), Kiosk/WC Methfesselstr. 26 (Pacht), Mehringdamm 90 Wilhelmshöhe, sowie Dora-Duncker-Park (Tempelhof-Schöneberg). Zuständig demnach das Bezirksamt Friedrichshain-Kreuzberg, Straßen- und Grünflächenamt (Berlin.de-Seite "Genehmigung für Sportkurse im Park" gefunden; Antragsweg für OL nicht geprüft).
- Liegenschaftsplan: 60 Flurstücke im Eigentum Land Berlin (`sch`); Anteil an der Bbox nicht berechnet.
- Denkmale: 6 Treffer vom Typ Baudenkmal (Links in die Denkmaldatenbank Berlin); ob der Park als Gartendenkmal erscheint, wurde nicht geprüft.
- Schutzgebiete Berlin (`h_nsg`): 0 Treffer. `i_lsg` bei einem Mini-Request 0 Treffer (resultType=hits). BfN: nicht erneut mit Treffer getestet.
- Nicht bestimmbar: Straßenverkehrsbehörde (Bezirk oder Verkehrslenkung Berlin, nicht verifiziert), Privateigentümer der Wohnbebauung am Rand, Wohnungsbaugesellschaften.
- Attribution laut vorhandener `ATTRIBUTION.txt`: Geoportal Berlin, dl-de/zero-2-0, plus OSM ODbL.

**Cochem (Rheinland-Pfalz)**: nicht live abgefragt. Zu erwarten: Gemeinde Stadt Cochem / Verbandsgemeinde Cochem / Kreis Cochem-Zell via VG250 (nicht verifiziert), Schutzstatus via BfN (Weinberge/Mosel: LSG, FFH wahrscheinlich, unverifiziert), Denkmal- und Wasserschutz über RLP-WFS (Wasserschutzgebiete WFS aus Katalog), Eigentümer nicht offen (RLP: ALKIS ohne Eigentümer).

---

# Teil C: Design für freemapper

## C1. Modell

Neuer Ausgabeabschnitt `permits` (Arbeitsname `info`):

```
area: polygon + CRS + Erstellzeit
authorities[]  { role, name, level, url, source, confidence }
protected_areas[] { type, name, source, status/leg_date, share_percent, area_m2, url, data_stand }
forest_units[]    { kind (Forstamt/Revier/Staatswald/...), name, share_percent, source }
landowners[]      { name/class, source, share_percent, basis ("Open Data"|"OSM"|"unbestimmbar") }
disclaimers, attributions[]
```

Ermittlung: Fläche mit allen Layern schneiden (WFS mit Bbox, dann lokaler Polygonschnitt, Anteil in Prozent), Treffer sortieren nach Anteil, Namen aus Attributen (`name` bei BfN, `namenr` bei Berlin-Grünanlagen). Zuständige Stelle folgt aus Gemeinde (VG250) und Kartenart (Wald: Landesforst/Untere Forstbehörde/UNB; Sprint: Ordnungsamt, Straßenverkehrsbehörde, Grünflächenamt).

## C2. Ausgabeorte (Empfehlung)

1. `permits.md` (DE/EN je `--lang`) neben der Karte, plus Maschinenlesbares `permits.json` (Basis für alles andere). Erster Schritt.
2. Abschnitt im Projektfile (nur Verweis/Hash und Stand, damit .omap nicht verändert wird).
3. MCP-Tool `permits_for_area` (nutzt dasselbe JSON). Unabhängig von KI (ADR 0037).
4. Optional eine Seite im Kandidaten-PDF, nur kurzer Hinweis plus Verweis auf `permits.md`; dort wegen Credit-Zeile Platz knapp.

## C3. Adapter-Passung (docs/superpowers/specs/2026-11-10-region-adapters-design.md)

Deklarativ, ein Abschnitt `[[info_layer]]` je Manifest, ohne neuen Code:

```toml
[[info_layer]]
id = "forest_districts"          # forest_units
kind = "wfs"
url = "https://www.brandenburg-forst.de/geoserver/IFUEK/ows"
type_name = "IFUEK:AM.ForestManagementArea"
crs = "EPSG:25833"
role = "forest_unit"
name_field = "..."               # nach DescribeFeatureType
license = "dl-de/by-2-0"
attribution = "Landesbetrieb Forst Brandenburg"
authority_template = "Landesbetrieb Forst Brandenburg, Oberförsterei {name}"
```

Gleiches Schema für WMS-only-Dienste (nur Info über GetFeatureInfo, kein Polygonschnitt; Anteil nicht möglich, deshalb Typ "nur Treffer"), für Shapefile-Download (NRW, einmaliger Download je Land, bewusst nicht Bulk) und für OSM-Filter. Generisch (ein Manifest für alle): BfN-Schutzgebiete, VG250, OSM-Tags. Je Land nötig: Forst-Layer, Waldfunktionen, Wasserschutz, Denkmal, Grünanlagen/Liegenschaften.

## C4. Aufwand (grobe Schätzung, nicht gemessen)

| Baustein | Aufwand | Wert |
|---|---|---|
| Generischer Info-Layer-Mechanismus (WFS lesen, Polygonschnitt, Anteile, `permits.md`/`.json`, Disclaimer, Attribution) | mittel | Voraussetzung für alles |
| BfN NSG/LSG/FFH/VSG/NP/Naturpark/BR + VG250 | klein, ein Manifest | deckt alle 11 Länder (Naturschutz, Gemeinde, Kreis) |
| OSM `protect_class`/`operator`/`owner`/`admin_level` | klein, Geofabrik bereits da | Ergänzung, Wasserschutz/Naturwald lückenhaft |
| Forst-Layer: BB, BE, NRW, MV(WMS), SN(WMS), ST(WMS), RP | je klein (1 Manifestabschnitt je Dienst) | Forstamt/Revier-Name; WMS-only braucht GetFeatureInfo-Pfad |
| Waldfunktionen (BB, HH, RP, SN) und Wasserschutz (HH, RP) | je klein | Totalreservat/Naturwald/Erholungswald, WSG |
| Berlin-Sprint-Paket (Bezirke, Grünanlagen, Liegenschaftsplan, Denkmale, Schutzgebiete) | klein, alles verifiziert | ein vorzeigbares Beispiel |
| Hessen, Niedersachsen, Schleswig-Holstein Forst | Recherche offen, evtl. nicht verfügbar | Hinweistext "nicht automatisch ermittelbar, bei Landesforst nachfragen" |
| Waldbesitzarten | nur NRW-Staatswald/Berlin; sonst nicht lösbar | im Report ehrlich als Lücke |

## C5. Grenzen und Pflichttexte

Jeder Report beginnt mit: "Hinweis, keine Rechtsauskunft. Maßgeblich sind die zuständigen Behörden. Datenstand und Genauigkeit siehe Tabelle; Flächen können fehlen oder veraltet sein (BfN: jährliche Zusammenführung, 'nicht für Planungszwecke geeignet')." Pflichtfelder: Datenstand je Quelle (BfN Layer-Stand aus dem Abstract), Abrufdatum, Lizenz und Quellenvermerk je Quelle (BfN: GeoNutzV-Hinweis, VG250: "© BKG (Jahr) dl-de/by-2-0", Brandenburg: "Landesbetrieb Forst Brandenburg", Hamburg: BUKEA-Text, Berlin: dl-de/zero-2-0, OSM: ODbL). Lizenz-Klärungsbedarf: MV (WMS ohne offene Lizenz), Sachsen-Anhalt (nur Copyright), Hessen/SH/NI (nichts gefunden), BfN (kein dl-de-Vermerk, nur Nutzungsbestimmungen Bund). Keine Eigentümernamen natürlicher Personen ausgeben. Keine Behörden-E-Mail-Adressen in Reports mitschleppen, nur Stellen-Webseiten.
Technisch: BfN, RP-Forst (`gis.wald.rlp.de`) blockieren evtl. Standard-HTTP-Clients (403 bei curl; WebFetch lieferte für BfN Daten). freemapper sollte einen ehrlichen User-Agent ohne E-Mail senden und HTTP 403 als "nicht abrufbar" melden.

## C6. Empfohlene Reihenfolge

1. Generisches `permits`-Gerüst plus BfN-Schutzgebiete, VG250 und OSM-Tags (alle Länder, ein Manifest, kleinster Aufwand, größter Nutzen).
2. Forst-Layer der verifizierten Länder: Brandenburg (Reviere + Waldfunktionen mit Totalreservat/Naturwald), NRW (Forstamt, Betriebsbezirk, Staatswald), Berlin (Forstämter), dann MV/SN/ST (WMS).
3. Berlin-Sprintpaket (Bezirke, Grünanlagen, Liegenschaftsplan, Denkmale, Landes-Schutzgebiete) als erstes Sprint-Beispiel.
4. Wasserschutzgebiete (HH, RP), RLP-Forstamt nach Verifikation, HE/NI/SH nachrecherchieren (offene Gaps).
5. Waldbesitz und Eigentümer: nur wo offen (NRW-Staatswald, Berlin Liegenschaftsplan), sonst Hinweistext.

## C7. Quellenliste (Auswahl)

- BfN WFS/WMS: https://geodienste.bfn.de/ogc/wfs/schutzgebiet, https://geodienste.bfn.de/ogc/wms/schutzgebiet
- BKG VG250: https://sgx.geodatenzentrum.de/wfs_vg250, Nutzungsbedingungen https://sgx.geodatenzentrum.de/web_public/gdz/lizenz/deu/nutzungsbedingungen_vg250.pdf
- Brandenburg: https://www.brandenburg-forst.de/geoserver/IFUEK/ows, .../INWFK/ows, .../IWFK/wms, .../ogc/fgk; Metadaten https://www.metaver.de/trefferanzeige?docuuid=A0EB03C2-0E64-4923-894A-D4A6B1B2AACB
- Berlin: https://gdi.berlin.de/services/wfs/{forstverwalt, schutzgebiete, liegenschaftsplan, landesgrundvermoegen, gruenanlagen, denkmale, alkis_bezirke}
- Sachsen: https://www.forsten.sachsen.de/kartendienste/wald/MapServer/WMSServer, Übersicht https://www.sbs.sachsen.de/forstliche-kartendienste-18448.html
- Sachsen-Anhalt: https://geodatenportal.sachsen-anhalt.de/ows_INSPIRE_LZW_US
- MV: https://www.geodaten-mv.de/dienste/forst_struktur_wms, forst_grundkarte_wms (WFS passwortgeschützt)
- NRW: https://www.wms.nrw.de/umwelt/waldNRW, https://www.opengeodata.nrw.de/produkte/umwelt_klima/wald_forst/wald/
- Hamburg: https://geodienste.hamburg.de/HH_WFS_Wald_in_Hamburg, HH_WFS_Waldfunktionen, HH_WFS_Schutzgebiete, HH_WFS_Wasserschutzgebiete
- RLP (Katalog): https://www.geoportal.rlp.de/spatial-objects/676 u. a.
