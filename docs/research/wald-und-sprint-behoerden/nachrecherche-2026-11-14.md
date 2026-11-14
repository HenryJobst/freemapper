# Nachrecherche: Forstverwaltung, Waldfunktionen, Wasserschutz und Sprint-Daten je Bundesland

Stand: 2026-11-14. Ergänzt [`waldschutz-forstbehoerden-recherche.md`](waldschutz-forstbehoerden-recherche.md).
Drei Recherchen mit zusammen rund 230 lesenden Anfragen, alle mit dem festen User-Agent des Programms und ohne
personenbezogene Daten. "ja" heißt: selbst abgerufen (Capabilities und mindestens ein Feature oder die
Attributliste); "teilweise": aus Katalog oder Suche; "nein": nur ein Hinweis. Alle Angaben sind Hinweise für die
Planung, keine Rechtsauskunft.

## Bundesweit

| Quelle | Dienst | Inhalt | Lizenz | verifiziert |
|---|---|---|---|---|
| BfG Trinkwasserschutzgebiete | ArcGIS REST `https://geoportal.bafg.de/arcgis1/rest/services/INSPIRE/AM/MapServer/1` (`AM.DrinkingWaterProtectionArea`, GeoJSON, höchstens 2000 je Abruf); Layer 2 Heilquellenschutzgebiete | 27 591 Gebiete; `Name`, `thematicIdentifier`, `zoneType`, keine Zonen I/II/III | GeoNutzV, Quellenvermerk "WasserBLIcK/BfG & Zuständige Behörden der Länder Datum" | ja |
| BKG DLM250 Straßen | WFS `https://sgx.geodatenzentrum.de/wfs_dlm250`, `dlm250:objart_42003_l`, GeoJSON | `wdm` (Widmung), `bez` ("B1"); 1:250 000, Kreis- und Gemeindestraßen fehlen teils | dl-de/by-2-0, "© GeoBasis-DE / BKG (Jahr) dl-de/by-2-0" | ja |

Für die Straßenklasse ist OSM (`ref` mit A, B, L, S, K) im ohnehin gelesenen Auszug genauer als DLM250.

## Je Bundesland

Lizenztexte stehen wörtlich, so wie die Dienste sie angeben.

### Hamburg (alles offen, GeoJSON)

`https://geodienste.hamburg.de/<Dienst>`, WFS 2.0.0, `application/geo+json`, `SRSNAME=EPSG:4326`, Präfix
`de.hh.up:`. Lizenz überall "Datenlizenz Deutschland Namensnennung 2.0, Quellenvermerk: Freie und Hansestadt
Hamburg, <Behörde>". Keine Personendaten in den gelesenen Beispielen. Verifiziert: ja.

| Dienst, Typ | Attribute | Behörde im Vermerk |
|---|---|---|
| `HH_WFS_Verwaltungsgrenzen`, `bezirke` (7) | `bezirk`, `bezirk_name` | Landesbetrieb Geoinformation und Vermessung |
| dito `stadtteile`, `ortsteile` | `stadtteil_name`, `stadtteil_schluessel` | dito |
| `HH_WFS_Wald_in_Hamburg`, `wald_stadtwald_hamburg` (8) | `bezirk`, `foersterei`, `bearb_stand` | BUKEA |
| dito `bannwald_stadtwald_hamburg`, `bannwald_heuckenlock`, `uebriger_wald_hamburg` | `bezirk`, `name` | BUKEA |
| `HH_WFS_Waldfunktionen`, `waldfunktionen_erholungswald` | `erhl_text` | BUKEA |
| `HH_WFS_Wasserschutzgebiete`, `wasserschutzgebiete` | `gebietsname`, `schutzzone`, `rechtsgrundlage` | Behörde für Umwelt und Energie |
| `HH_WFS_Gruenplan`, `verzeichnis_oeffentlicher_gruenanlagen` (3579) | `benennung`, `gruen_art`, `bezirk`, `eigentum`, `verwaltungsvermoegen`, `bekanntmachung` | Behörde für Umwelt und Energie |
| `HH_WFS_Denkmalschutz`, `flaechen_gartendenkmale`, `ensembles`, `gebaeude` | `kdstatus`, `bezeichnung`, `info` | zuständige Behörde |
| `HH_WFS_Strassen_und_Wegenetz`, `strassennetz_gesamt` | `klasse`, `strassenschluessel`, `strassenname` | BVM |

### Saarland (alles offen, GeoJSON)

`https://geoportal.saarland.de/arcgis/services/Internet/<Name>/MapServer/WFSServer`, WFS 2.0.0, `GEOJSON`,
EPSG:25832. Katalog: CC BY 4.0, "Quellenvermerk: © GDI-SL (Jahr)"; Capabilities "Es gelten keine Bedingungen".
Verifiziert: ja.

- `Forstwirtschaft`: `Reviergrenze_Staatswald` (`NUMMER`, `NAME`, `LINK` zur SaarForst-Revierseite),
  `Abteilungsgrenze_Staatswald`, `Gesamtwald_SL` (`BETRIEB`), `Naturwaldzellen`. Eine Forstamtsebene gibt es nicht.
- `Schutzgebiete_INS`: `Wasserschutzgebiet` (`NAME`, `SCHUTZZONE`, `VERORDNUNG`, `URL`), `Naturwaldzellen`,
  `Urwald_SL`, `Naturdenkmal_Flaechen`.
- `Denkmalschutz`: `Baudenkmale` (Punkte, Link zur Denkmalliste), `Bodendenkmal`, `Grabungsschutzgeb`.
- Das ATKIS Basis-DLM ist im Saarland kostenpflichtig.

### Brandenburg (Ergänzung)

WFS `https://www.brandenburg-forst.de/geoserver/IWFK/ows`, GeoJSON, EPSG:25833, Fees und AccessConstraints "NONE".
Verifiziert: ja. `wfk_nwr` Naturwald (`name`, `status`, `rechtsgrund`), `lfu_trv` Totalreservat (`sg_name`,
`schutzstat`), `wfk_nbt` nicht betretbare Fläche (`rechtsgrund`), `wfk_ewf`/`wfk_ewr` Erholungswald,
`bldam_gdm_fl` Gartendenkmale (`ort`, `listentext`). Bodendenkmale nur mit `sortBy`.

### Berlin (Ergänzung)

WFS `https://gdi.berlin.de/services/wfs/wsg`, `wsg:wsg`, GeoJSON: `wasserwerk`, `zone`, `verordnung`,
`veror_link`. Lizenz wie die anderen Berliner Dienste (dl-de/zero-2-0). Verifiziert: ja.

### Nordrhein-Westfalen (Ergänzung)

- WFS `https://www.wfs.nrw.de/umwelt/linfos`, `GEOJSON`, EPSG:25832: `wfs_linfos:nwz_polygon` Naturwaldzellen,
  `wfs_linfos:weg_polygon` Wildnisentwicklungsgebiete (mit `regionalforstamt`). Lizenz laut WMS-Capabilities
  dl-de/by-2-0, "Land NRW". Verifiziert: ja.
- Wasserschutzgebiete nur als Shapefile (`opengeodata.nrw.de/.../wasserschutzgebiete/`, 15 MB, dl-de/zero-2-0).

### Schleswig-Holstein

WFS `https://umweltgeodienste.schleswig-holstein.de/WFS_UWAT`, nur GML 3.2.2: `app:shlf_gesamtflaeche` Landeswald
der SHLF (1410 Flächen, `revier`, `reviername`), CC BY 4.0, "Quellenvermerk: LfU SH". Dazu `app:shlf_naturwald`,
`app:wsg_zonen`. Untere Forstbehörde: das LLnL, Abteilung Fischerei und Forst. Verifiziert: ja (Typen und
Lizenz; Attribute von `wsg_zonen` nicht). Kiel: WFS mit Ortsbeiräten und Grünanlagen (CC BY 4.0).

### Baden-Württemberg

- LUBW Wasserschutzgebiete: `https://rips-gdi.lubw.baden-wuerttemberg.de/arcgis/services/wms/UIS_0100007500200036_AGGR/MapServer/WFSServer`,
  `Wasserschutzgebietszone_M1`, GeoJSON: `WSG`, `ZONE`, `STATUS`, `DST` (zuständiges Landratsamt). Lizenz laut
  Metadaten dl-de/zero-2-0 (nur Suchergebnis). Verifiziert: ja (ein Feature).
- ForstBW Staatswald (`owsproxy.lgl-bw.de`, INSPIRE, nur GML): Forstbezirk nur als Nummer.
- FVA-Waldfunktionen nur unter Nutzungsbedingungen der FVA (nicht offen); Bannwald und Schonwald nur auf Anfrage.

### Bayern

- LfU Schutzgebiete (WFS, GeoJSON, CC BY 4.0): doppelt zum BfN.
- Wasserschutzgebiete nur als Zip ohne Zonen (CC BY 4.0).
- Waldfunktionsdienste der Forstverwaltung (fovgis/fov-web) liefern 404.
- Zuständigkeitsbereiche der AELF (untere Forstbehörde) nur als Katalogeintrag, Zugang nicht gefunden.
- ATKIS Basis-DLM offen (CC BY 4.0): `AX_Strasse.widmung`.

### Hessen

Kein Forstamtslayer. Wasserschutzgebiete: `https://geodienste-umwelt.hessen.de/arcgis/services/inspire/bewirtschaftungsgebiete/MapServer/WFSServer`,
`TWS_HQS_ALK`, GeoJSON, `ZONE`; CC BY 4.0, Quellenvermerk HLNUG.

### Niedersachsen

Kein Forstamtslayer; Schutzgebiete-WFS beim NLWKN (doppelt zum BfN); Wasserschutzgebiete landesweit ohne WFS.

### Bremen

Stadtbezirke und Stadtteile: `https://geodienste.bremen.de/wfs_verwaltungsgrenzen` (nur GML, CC BY 4.0,
"Landesamt GeoInformation Bremen"). Wasserschutzgebiete: ArcGIS-WFS auf `gis-hub.bremen.de`, GeoJSON. Kein Wald.

### Sachsen, Sachsen-Anhalt, Mecklenburg-Vorpommern, Rheinland-Pfalz

- Sachsen: Forstbezirke, Reviere und Waldfunktionen (Erholungswald, Naturwaldzellen, Trinkwasserschutzzonen im
  Wald) nur als WMS von Sachsenforst; GetFeatureInfo liefert `application/geojson` ohne Geometrie; dl-de/by-2-0.
- Sachsen-Anhalt: Forstämter nur als WMS, Lizenz nur "© LZW LSA, 2015".
- Mecklenburg-Vorpommern: Forst-WMS mit "Urheberrecht" und Personendaten im Layer der Ämter; der Forst-WFS ist laut
  Capabilities passwortgeschützt und nur auf Anfrage (nicht nutzen). Wasserschutzgebiete WFS mit GeoJSON unter
  CC BY-SA.
- Rheinland-Pfalz: `gis.wald.rlp.de` antwortet 403; kein anderer Weg zu den Forstamtsgrenzen gefunden.

## Folgerung für freemapper

1. Hamburg: Bezirke mit Bezirksamt, Grünanlagen mit Bezirk und Eigentum, Revierförsterei im Stadtwald, Bannwald
   und Erholungswald, Wasserschutzgebiete mit Zone, Denkmale. Ein Dienstmuster, alles GeoJSON.
2. Wasserschutzgebiete mit Zone und Waldfunktionen (Naturwald, Totalreservat, nicht betretbar, Erholungswald,
   Bannwald, Wildnisentwicklung) in den Bundesländern mit offenen GeoJSON-Diensten: Berlin, Brandenburg, NRW,
   Saarland, Hessen, Baden-Württemberg, Bremen.
3. Saarland: Reviere der SaarForst, Staatswald, Baudenkmale.
4. Schleswig-Holstein: Landeswald mit Revier (GML).
5. Sachsen über GetFeatureInfo (Treffer an Punkten, keine Flächenanteile).
6. Bundesweit als Rückfall: die Trinkwasserschutzgebiete der BfG ohne Zonen; die Straßenklasse aus OSM.
