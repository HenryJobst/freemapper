# Schweden: Dienste, Lizenzen und was ein Adapter bräuchte

Recherche vom 2026-11-16 für einen möglichen Regionsadapter für Schweden, die erste Region außerhalb
Deutschlands auf dem neuen Schema. Alle Anfragen wurden mit `curl` und ohne personenbezogene Daten gestellt.
**Es wurde kein Konto angelegt**: alles, was eine Anmeldung braucht, ist von außen aufgenommen, das
authentifizierte Innenleben ruht also auf den veröffentlichten Dokumenten und den Antworten der Server auf
unauthentifizierte Anfragen, nicht auf einem eigenen Download.

**[live]** wurde an diesem Tag von diesem Rechner abgefragt (HTTP-Status und Antwortkörper). **[doc]** stammt aus
einer amtlichen Seite oder aus den Metadaten des Dienstes. **[unverified]** markiert Aussagen aus Erinnerung oder
Sekundärquellen.

Nachtrag vom 2026-11-18: Ein schwedischer Orientierungsläufer berichtet aus eigener Nutzung, dass ein Konto
nötig ist, dass sich je Vorgang nur wenige Kacheln herunterladen lassen und dass das vor etwa einem Jahr
deutlich schlechter geworden ist. Der Bericht steht in §2, seine Folge für einen Adapter in §9, die offene
Frage dazu in §10.

Behörde: Lantmäteriet (Kartografie, Kataster und Grundbuch). Vertrieb: Geotorget
(`geotorget.lantmateriet.se`) und drei STAC-Kataloge auf `api.lantmateriet.se`. CRS: SWEREF99 TM
(EPSG:3006), Höhen in RH2000, beides zusammen als EPSG:5845. Geofabrik-Extrakt: `europe/sweden`.

---

## 0. Kurzantwort

* **Kostenlos, aber nicht offen wie die deutschen Länderdienste.** Lantmäteriet nennt seine kostenlosen
  Produkte *värdefulla datamängder* („wertvolle Datensätze“, nach der EU-Richtlinie über hochwertige
  Datensätze), nicht *öppna data*. Eine Person oder eine Organisation braucht ein Konto, ein *systemkonto*, eine
  Berechtigung je Produkt und für mehrere Produkte einen geprüften Zweck und akzeptierte Bedingungen. Downloads
  tragen ein OAuth2-Bearer-Token [doc], und der Asset-Host antwortet ohne eines mit `401` [live]. Es gibt keine
  schlüsselfreie Download-URL und kein schlüsselfreies WMS für Höhen, Orthofoto oder Vektoren. Nach dem Bericht
  eines schwedischen Orientierungsläufers lassen sich außerdem je Vorgang nur wenige Kacheln laden
  [unverified] — für eine Karte heißt das Handarbeit (§2). Schlüsselfrei sind dagegen Bodenbedeckung und
  Objekthöhe (NMD, 10 m, CC0) und die Schutzgebiete (§7). Und nicht alles ist kostenlos: Höhen, Vektoren und
  Orthofoto sind *avgiftsfrei*, das **Oberflächenmodell kostet** (§1).
* **Die Metadaten sind schlüsselfrei und brauchbar.** Vier STAC-Kataloge antworten ohne Token, mit Footprint,
  Auflösung, CRS, Dateigröße und einer direkten Asset-URL je Kachel [live], und das API-Portal nennt alle
  Schnittstellen samt Kontingent (§4.1). Ein Adapter kann einen schwedischen Download *planen*, ohne
  Zugangsdaten, und scheitert erst beim Holen.
* **Die Formate passen nicht.** Höhen und Orthofoto sind Cloud-Optimized GeoTIFFs (`image/tiff;
  profile=cloud-optimized`) [live], die Vektoren im Zip als GeoPackage [live, nach ihrer Beschreibung] —
  **Gebäude nur so**, Bodenbedeckung, Verwaltungseinheiten, Hydrografi und Grundstücksgrenzen dagegen auch als
  **GeoJSON** über die OGC-Features-API (§4.1). freemapper liest gekachelte **LZW**-Float32-TIFFs per Range und
  hat keinen GeoPackage-Leser, aber einen GeoJSON-Leser für WFS-2.0-Dienste (§8.2); die schwedischen COGs sind
  **deflate**, das Orthofoto 8-Bit-RGB(I) [live].
* **Das CRS passt.** SWEREF99 TM benutzt die Parameter von UTM 33 (GRS80, Mittelmeridian 15° O, k0 0,9996,
  falscher Rechtswert 500 000, kein falscher Hochwert); seine Koordinaten lassen sich also unverändert als
  EPSG:25833 verwenden, und die 10-km-Kacheln des DTM liegen auf freemappers Kilometergitter. Nur das Höhendatum
  ist ein anderes (RH2000).
* **Empfehlung:** kein Manifest. Ein schwedischer Adapter braucht ein Token im Fetcher, einen
  deflate-COG-Leser, einen GeoPackage-Leser und eine Antwort auf die Regel „keine Raster, keine Karte“. Bis
  dahin kann Schweden keine Region im Sinne der 15 deutschen sein — siehe §8 und §9.

---

## 1. Was eine Orientierungskarte braucht und was Schweden bietet

| Kartenobjekt | Schwedisches Produkt | Schlüsselfrei | Format | CRS | Lizenz | Blocker |
|---|---|---|---|---|---|---|
| Gebäude | Byggnader (`stac-vektor`) | nein | GeoPackage im Zip | 3006 | CC BY 4.0 | Token, GPKG-Leser |
| Bodenbedeckung (Wald, Feld, Siedlung) | Marktäcke (`stac-vektor`) | nein | GeoPackage im Zip, auch GeoJSON | 3006 | CC BY 4.0 | Token, GPKG- oder OAPIF-Leser |
| Bodenbedeckung, grob | NMD (`lc-nmd`) | **ja** | GeoTIFF, 16 Bit, 10 m | 3006 | CC0 1.0 | 10 m grob, PackBits |
| Objekthöhe und -täckning (grober nDSM) | NMD tilläggsskikt | **ja** | vier Raster, 10 m | 3006 | CC0 1.0 | 10 m grob |
| Verwaltungseinheiten, Küstenlinie | Kommun, län och rike | nein | GeoPackage im Zip, auch GeoJSON | 3006 | CC BY 4.0 | Token, GPKG- oder OAPIF-Leser |
| Wege, Trassen, kleine Objekte | OSM (`europe/sweden`) | ja | PBF | WGS84 | ODbL | 982 MB Extrakt, kein Teil-Extrakt |
| Orthofoto | Ortofoto (`stac-bild`, 737 Sammlungen) | nein | COG, 0,16 m RGBI | 3006 | CC BY 4.0 **plus DSGVO-Prüfung** | Token, 8-Bit-COG-Leser, ~0,6 GB je Kachel |
| Gelände (Höhenlinien, Neigungen) | Markhöjdmodell (`dtm-cog`) | nein | COG, 1 m Float32 | 5845 | CC BY 4.0 | Token, deflate im Range-Leser |
| Oberflächenmodell (nDSM) | Ythöjdmodell (`dsm-cog`) | nein | COG, 0,25 m Float32 | 5845 | `other` | Token, Lizenz unklar |
| Bäume / Vegetationshöhe | Laserdata Skog (`dsm-skoglig-copc`) | nein | COPC LAZ | 5845 | `other` | Token, Lizenz unklar |
| Schutzgebiete | Naturvårdsverket WMS | **ja** | WMS 1.3.0 | 3006 | CC0 1.0 | keiner |

Die beiden `other`-Lizenzen sind die Lücke, die am meisten weh tut: ohne das Oberflächenmodell gibt es kein
nDSM, und ohne nDSM kann freemapper einen Park nicht von einem Wald unterscheiden, kann die Baumsignaturen der
Trainingskarte nicht setzen, und die Regel „gelb, wo keine Bäume stehen“ der `-training`-Karte hat nichts zu
lesen. Die deutschen Adapter haben alle ein DOM.

**Gebühr und Rechtsprüfung** stehen je Produkt in der öffentlichen Produktübersicht von Geotorget [live]:
*avgiftsfrei* sind Markhöjdmodell (1 m und grid 50+), Marktäcke, Byggnad, Kommun/län/rike, Ortnamn, Hydrografi,
Laserdata Skog, Topografi 10/50/100/250/1M, die topografischen Rasterkarten und die historischen Karten;
**kostenpflichtig** sind **Ythöjdmodell** (beide Varianten), Laserdata NH, die digitalen Flugbilder und die
meisten Visningstjänste. Eine *juridisk prövning* verlangen unter anderem Byggnad, Ortofoto,
Fastighetsindelning, Belägenhetsadress, Hydrografi Nedladdning und Topografi 10 — Markhöjdmodell und Marktäcke
nicht. Das erklärt die `other`-Lizenz des Oberflächenmodells: es ist das eine Produkt, an dem das nDSM hängt,
und es kostet Geld.

---

## 2. Zugang: Geotorget, Konten und was „kostenlos“ bedeutet

* Lantmäteriets Seite *Avgiftsfria produkter* [doc] sagt, dass die EU-Richtlinie über hochwertige Datensätze
  mehr Produkte kostenlos gemacht hat und dass sie *värdefulla datamängder* heißen. Der Zugang braucht ein
  privates Konto oder ein Organisationskonto mit einem *systemkonto* (Maschinenkonto für programmatische
  Nutzung) und *behörighet* (Berechtigung) je Produkt, dazu die Annahme der Bedingungen.
* Dieselbe Seite [doc], wörtlich: „Vissa produkter kräver ändamålsprövning enlighet fastighetsregisterlagen,
  vissa produkter innehåller personuppgifter och kräver godkännande av villkor samt att du anger hur du ska
  behandla personuppgifter.“ — Einige Produkte brauchen eine Zweckprüfung nach dem Grundbuchgesetz, einige
  enthalten personenbezogene Daten und brauchen akzeptierte Bedingungen und eine Angabe, wie diese Daten
  behandelt werden.
* Die Seite *Öppna data* [doc] trennt davon eine zweite, kleinere Menge von *öppna data* unter **CC0**, wo
  Namensnennung geschätzt, aber nicht gefordert ist. Keines der Produkte aus §1 gehört dazu.
* Der Produktkatalog ist öffentlich: `geotorget.lantmateriet.se/geodataprodukter` listet 79 Produkte mit
  Zugangsart, Format, Endpunkt und je einem Vermerk **Avgift** (Gebühr) und **Juridisk prövning**
  (Rechtsprüfung) [live]. Der alte Pfad `/sv/geodata/vara-produkter/produktlista/` leitet mit `301` genau auf
  diese Seite um [live], ist also nur ein anderer Name dafür. Preise und Bedingungen stehen auch dort nicht —
  was ein Produkt kostet, ist erst nach der Anmeldung zu lesen.
* Ein schwedischer Orientierungsläufer berichtet aus eigener Nutzung (mündlich, 2026-11-18), dass man sich ein
  Konto anlegen muss und je Vorgang nur wenige Kacheln herunterladen kann, und dass das vor etwa einem Jahr
  deutlich schlechter geworden ist [unverified]. Das deckt sich mit dem, was die Server von außen zeigen (Konto
  und Token, §4). Für eine Karte zählt es: 4 km Kantenlänge berühren bis zu neun Orthofoto-Kacheln à 0,6 GB
  und bis zu neun DSM-Kacheln (§3.5); bei „ein paar Kacheln zur Zeit“ wird der Bezug zur Handarbeit. Ob die
  Grenze die Weboberfläche von Geotorget betrifft oder auch die API-Assets unter `dl1.lantmateriet.se`, ist offen
  (§10); die Kontingente im API-Portal (§4.1) sprechen gegen ein API-Limit — davon hängt ab, ob ein Adapter
  überhaupt unbeaufsichtigt laden kann.
* **Die Anmeldung hängt an einer schwedischen Identität.** „Mina sidor“, der Kontobereich für Grundstücks- und
  Vermessungsvorgänge, meldet mit **BankID** oder **Freja+** an; BankID setzt eine schwedische Personnummer
  voraus, Freja+ eine samordningsnummer [live]. Welchen Weg Geotorget selbst nimmt — dieselbe e-legitimation,
  ein Organisationskonto oder beides —, ist von außen nicht zu sehen und steht in §10. Für eine Person oder
  einen Verein ohne schwedische Kennung ist das eine Hürde vor der ersten Kachel, nicht erst beim Token.
* **Geotorget selbst ist öffentlich.** Die Startseite, die Produktübersicht, der *Produktstöd* (Anleitungen)
  und der Zugang zur Nationalen Geodatenplattform sind ohne Anmeldung erreichbar [live]; verschlossen sind
  Bestellung, Bedingungen und Download.

* **Ein Konto allein öffnet nichts.** Nach dem Anlegen eines Kontos antwortet das Developer Portal mit
  `Error 403 : Forbidden — You don't have sufficient privileges to access the Developer Portal` (Bericht vom
  2026-11-19). Das ist die Rollenprüfung von WSO2, nicht ein falsches Passwort: dem Konto fehlt die
  Subscriber-Rolle. Lantmäteriets Anleitung sagt, was dann zu tun ist — wer ein API nicht sieht, bestellt die
  Berechtigung bei Geodata-support (`geodatasupport@lm.se`) [doc]. Zwischen „Konto angelegt“ und „erste
  Kachel“ liegt also ein Mailwechsel.

**Folge für einen Adapter.** Ein schwedischer Adapter bräuchte eine Zugangsberechtigung, die eine Person für ihr
eigenes Konto beschafft, aufbewahrt außerhalb des Projektordners und niemals in `freemapper.toml`,
`ATTRIBUTION.txt` oder einer Fortschrittszeile landet. Das ist eine neue Art von Zustand für freemapper, das
bisher nirgends ein Benutzerkonto braucht.

---

## 3. Die STAC-Kataloge

Vier Wurzeln, alle schlüsselfrei [live]:

| Katalog | URL | Inhalt |
|---|---|---|
| Höjd | `https://api.lantmateriet.se/stac-hojd/v1` | Gelände- und Oberflächenmodelle, Laserdaten |
| Vektor | `https://api.lantmateriet.se/stac-vektor/v1` | Gebäude, Bodenbedeckung, Adressen, Verwaltungseinheiten |
| Bild | `https://api.lantmateriet.se/stac-bild/v1` | Orthofoto, 737 Sammlungen |
| Karta | `https://api.lantmateriet.se/stac-karta/v1` | Topowebb als Rasterpaket, zwei Militärkarten (§3.7) |

Jeder hat `/collections`, `/collections/{id}`, `/collections/{id}/items`, `/collections/{id}/items/{item}`,
`/search`, `/queryables` und `/conformance` [live]. Das OpenAPI-Dokument von `stac-hojd`
(`/stac-hojd/v1/api`, OpenAPI 3.1.0) nennt **kein Sicherheitsschema** [live], die Metadaten sind also öffentlich
gedacht, der Asset-Host nicht.

### 3.1 Höjd: Gelände, Oberfläche, Laser

| Sammlung | Titel | Lizenz | Objekte |
|---|---|---|---|
| `dtm-cog` | Markhöjdmodell | `CC-BY-4.0` | 10-km-Kacheln, 1 m |
| `dsm-cog` | Ythöjdmodell | `other` | 2,5-km-Kacheln, 0,25 m |
| `dsm-skoglig-copc` | Laserdata Skog | `other` | 2,5-km-COPC-Kacheln |

### 3.2 Vektor

| Sammlung | Titel | Lizenz | Granularität |
|---|---|---|---|
| `byggnader` | Byggnader | CC-BY-4.0 | eine je Kommun |
| `marktacke` | Marktäcke | CC-BY-4.0 | eine je Kommun |
| `fastighetsindelning` | Fastighetsindelning | CC-BY-4.0 | eine je Kommun |
| `belagenhetsadresser` | Belägenhetsadresser | CC-BY-4.0 | eine je Kommun |
| `ortnamn` | Ortnamn | CC-BY-4.0 | eine je Kommun |
| `kommun-lan-rike` | Kommun, län och rike | CC-BY-4.0 | `aktuell`, `2026`, `2025`, `2020`, `2010` |

Jede Beschreibung sagt „En geopackagefil som innehåller …“ [live], der Download ist also ein Zip um ein
GeoPackage, geliefert je Gemeinde und nicht je Kartenfläche. Stichproben [live]:

| Objekt | Asset | Bytes |
|---|---|---|
| `byggnader/1281` (Lund) | `https://dl1.lantmateriet.se/byggnadsverk/byggnad_kn1281.zip` | 10 958 002 |
| `byggnader/1285` (Eslöv) | `…/byggnad_kn1285.zip` | 3 798 837 |
| `marktacke/1281` | `https://dl1.lantmateriet.se/mark/marktacke/marktacke_kn1281.zip` | 7 573 446 |
| `kommun-lan-rike/aktuell` | `…/administrativ-indelning/kommun-lan-rike/kommun-lan-rike_aktuell.zip` | 6 000 798 |

`kommun-lan-rike` deckt das ganze Land ab (bbox 8,61° O–25,49° O, 54,97° N–69,03° N) und sagt „Uppdateras
veckovis" [live]. Es ist die naheliegende Quelle für die `boundary.geojson` des Adapters — nur dass es ein
GeoPackage ist und `boundary.file` GeoJSON will.

### 3.3 Bild: Orthofoto

737 Sammlungen [live], benannt `orto-<Gebiet>-<Jahr>` (`orto-a2-2026`, `orto-n2-2026`, … dazu benannte Gebiete
wie `orto-jokkmokk-<Jahr>`, `orto-stromsund-<Jahr>`, `orto-gallivare-<Jahr>`). Die Gebietsbuchstaben sind
Befliegungsgebiete; die `keywords` einer Sammlung tragen die Gemeindenamen [live]. Daneben: `orto-dsm-cog`
(„Sant ortofoto“, Lizenz `other`) und die historischen Serien.

Stichproben [live], alle EPSG:3006, 2,5-km-Kacheln, geflogen 2026:

| Objekt | Sammlung | Auflösung | Bytes |
|---|---|---|---|
| `o61775_3825_25_mr26` | `orto-a2-2026` | 0,16 m, RGBI | 595 979 964 |
| `o61950_4475_25_mr26` | `orto-a2-2026` | 0,16 m, RGBI | 641 599 081 |
| `o62575_4700_25_mr26` | `orto-a2-2026` | 0,16 m, RGBI | 727 817 048 |
| `o65500_5975_25_mr26` | `orto-n2-2026` | 0,16 m, RGBI | 746 591 174 |

Die Auflösung ist eine Eigenschaft (`upplosning`), keine Konstante: eine Sammlung kann andere Jahre und andere
Auflösungen enthalten. Ein Adapter muss sie je Objekt lesen und ablehnen, was er nicht abtasten kann.

Anders als die Höhenobjekte trägt ein Orthofoto-Objekt `proj:bbox` und `proj:code`: `o61775_3825_25_mr26` hat
`proj:bbox [382500, 6177500, 385000, 6180000]` und `proj:code EPSG:3006` [live], also 2,5 km und die
Südwest-Ecke der Kachel — genug, um eine Kachel ohne zweite Anfrage zu platzieren. Ein `proj:shape` fehlt, die
Pixelzahl ist aus `upplosning` zu rechnen.

### 3.4 Was ein Objekt sagt

Das Objekt einer DTM-Kachel, `collections/dtm-cog/items/617_38` [live]:

| Feld | Wert |
|---|---|
| `id` | `617_38` |
| `bbox` | 13,0881–13,2513° O, 55,6610–55,7532° N (WGS84) |
| `properties.proj:code` | `EPSG:5845` |
| `properties.hojdmodelltyp` | `markhöjdmodell` |
| `properties.geometriskupplosning` | `1.0` |
| `properties.lagesosakerhetplan` / `…hojd` | `0.3` / `0.1` |
| `properties.huvudsakliglagesbestamningsmetod` | `Luftburen laserskanning` |
| `properties.datetime` | `2025-09-29` |
| `assets.data` | `…/hojd/data/grid/mhm/61_3/m617_38.tif`, `image/tiff; …; profile=cloud-optimized`, 298 640 561 B |
| `assets.info` | `…/m617_38_info.json`, 4 340 B |
| `assets.metadata` | `…/m617_38_ursprung.json`, 195 305 B |
| `assets.thumbnail` | `…/m617_38_thumbnail.png`, 29 583 B |
| `assets.breakgeometry` | `…/m617_38_brytgeometri.gpkg`, 294 912 B |

Zwei Einzelheiten sind für einen Adapter wichtig:

* **Das Höhenobjekt trägt weder `proj:bbox` noch `proj:shape`** (ein Orthofoto-Objekt hat das erste, siehe
  §3.3). Die projizierte Ausdehnung, die Pixelgröße und der Nodata-Wert stehen im `info`-Asset [live]: `GTiff`,
  `float32`, Nodata `-9999`, 10 000 × 10 000 px, CRS `EPSG:5845`, Transform `[1.0, 0, 380000, 0, -1, 6180000]`,
  Blöcke 512 × 512, gekachelt, **deflate**, PROJJSON `CompoundCRS "SWEREF99 TM + RH2000 height"`. Ein Adapter,
  der einen Download allein aus dem Objekt plant, bräuchte je Kachel eine zusätzliche Anfrage für das
  `info`-Asset.
* **`breakgeometry` ist ein GeoPackage** mit den Bruchkanten des Geländemodells — eine brauchbare Zugabe für
  die Kandidaten, aber ein weiterer GeoPackage-Verbraucher.

Das Oberflächenmodell, `items/2026_a2-617_39_5000` [live]: `proj:code` `EPSG:5845`, `upplosning` `0.25`,
`hojdmodelltyp` `ythöjdmodell`, `insamlingsomrade` `2026_A2`, `upplosningflygbild` `0.15`, Asset
`…/hojd/data/grid/yhm/2026_a2/y2026_a2-617_39_5000_dsm.tif` mit 229 814 895 B, `info`-Asset mit 0,25 m und
Ursprung (390 000, 6 177 500), ein `pointmetadata`-Asset `…_class.tif` mit 1 321 508 B, Thumbnail 35 018 B.
Kein `breakgeometry` hier. Dass das Produkt **kostenpflichtig** ist, steht in keiner dieser Angaben, sondern
nur in der Produktübersicht von Geotorget (§1) [live].

Die Punktwolke, `25a005-61775_3850_25` [live]: `…/hojd/data/sls/25a005/y25a005-61775_3850_25.copc.laz`,
81 714 617 B, `pc:count` 16 464 575, `pc:density` 0,63 pts/m², EPSG:5845.

### 3.5 Kachelgitter und Benennung

| Produkt | Kachel | Pixel | Bytes je Kachel | Bytes je km² |
|---|---|---|---|---|
| DTM 1 m | 10 km | 10 000² | ~250–315 | ~2,5–3,2 MB |
| DSM 0,25 m | 2,5 km | 10 000² | ~230 | ~37 MB |
| Orthofoto 0,16 m | 2,5 km | 15 625² | ~0,6–0,75 GB | ~96–120 MB |
| Laserdata Skog | 2,5 km | — | ~82 | ~13 MB |

Namen: DTM `m<E/10000>_<N/10000>.tif` im Ordner `<floor(N/100000)>_3/`; DSM
`y<Gebiet>-<N/10000>_<E/10000>_<Eoffset/100><Noffset/100>_dsm.tif`; Orthofoto
`o<N/100>_<E/100>_25_mr<yy>.tif` [live]. Die Südwest-Ecke im Namen ist Lantmäteriets eigenes Indexsystem
und in HMK – Höjdmodell 2023 vorgeschrieben [doc], siehe §7.5.

Zur Größenordnung: eine Kartenfläche samt Puffer von höchstens 4 km je Seite (`MAX_AREA_SIDE_M` = 20 000 px ×
0,2 m) berührt **eine bis vier** DTM-Kacheln, **bis zu neun** DSM-Kacheln und **bis zu neun**
Orthofoto-Kacheln, also etwa 0,3–1,2 GB Gelände, 2,1 GB Oberflächenmodell und 5 GB Orthofoto, wenn die Dateien
ganz geladen werden.

### 3.6 Suche

`/search?collections=…&bbox=…&limit=…&fields=…` funktioniert [live]; über Lund antwortete es mit `617_38`
und `617_39` und deren Asset-URLs. Beobachtungen:

* Die Antwort hat `type`, `links`, `features`, `numberReturned` — **kein `numberMatched`** [live], die Zahl der
  Objekte hinter einer Abfrage ist also unbekannt, und geblättert werden muss über `links`.
* `sortby=-properties.datetime` wurde angenommen (`item-search#sort` steht in `/conformance`), die beiden
  gelieferten Orthofoto-Objekte kamen aber in **aufsteigender** Zeitfolge [live]. Auf die Sortierung ist kein
  Verlass.
* `fields` funktioniert; `/conformance` listet `item-search#fields`, `#query`, `#filter`, CQL2 basic/JSON/text
  [live].

### 3.7 Karta: Topowebb und die Militärkarten

Der vierte und kleinste Katalog, drei Sammlungen [live]:

| Sammlung | Titel | Lizenz |
|---|---|---|
| `topowebb` | Topografisk webbkarta Nedladdning, raster | `CC-BY-4.0` |
| `nmk250` | Nationell militär karta 1:250 000 | `other` |
| `nmk50` | Nationell militär karta 1:50 000 | `other` |

`topowebb` besteht aus zwei Objekten, `topowebb-3006` und `topowebb-3857`, und deren `data`-Asset ist ein
**GeoPackage**: 175 313 006 592 B für SWEREF99 TM, 156 273 930 240 B für Web Mercator [live]. Das ist ein
einziges Riesenpaket mit Rasterkacheln — der Code kann es weder öffnen (kein GeoPackage-Leser, §8.2) noch
laden (`MAX_DOWNLOAD_BYTES` 512 MB). Für eine Orientierungskarte trägt Topowebb ohnehin nichts bei: eine
fertige kartografische Karte mit Signaturen, kein Gelände- oder Oberflächenmodell.

---

## 4. Authentifizierung, wie beobachtet

| Anfrage [live] | Antwort |
|---|---|
| `GET https://dl1.lantmateriet.se/hojd/data/grid/mhm/61_3/m617_38.tif` | `401`, `www-authenticate: Basic realm="Authorization Server"` |
| dieselbe mit `Authorization: Bearer abc` | `401`, `Bearer error="invalid_token", error_description="Provided token isn't active"` |
| `POST https://api.lantmateriet.se/token` (`grant_type=client_credentials`, keine Client-Authentifizierung) | `401` `{"error":"invalid_client","error_description":"Unsupported client authentication mechanism"}` |
| dieselbe mit `-u abc:def` | `401` `{"error":"invalid_client","error_description":"Client credentials are invalid."}` |
| `GET https://api.lantmateriet.se/token` | `405` |
| `POST https://apimanager.lantmateriet.se/oauth2/token` (`grant_type=client_credentials`, keine Client-Authentifizierung) | `401` wie oben, `Unsupported client authentication mechanism` |
| dieselbe mit `-u abc:def` | `401`, `Client credentials are invalid.` |

Zusammengelesen: der Asset-Host will ein **Bearer**-Token (RFC 6750, im Stil eines WSO2 API Manager), der
Token-Endpunkt will eine **Client**-Berechtigung im `Authorization`-Header eines Formular-POST und antwortet in
der üblichen `invalid_client`-Form. Beide Adressen — `/token` auf `api.` und `/oauth2/token` auf `apimanager.` —
antworten gleich [live], die dokumentierte ist die zweite.

**Der Ablauf steht offen in der Portal-Dokumentation** (`https://apimanager.lantmateriet.se/docs/`, ohne
Anmeldung lesbar) [doc]: `POST https://apimanager.lantmateriet.se/oauth2/token` mit
`Authorization: Basic base64(ConsumerKey:ConsumerSecret)`, `grant_type=client_credentials` und
`Content-Type: application/x-www-form-urlencoded`; Antwort `access_token`, `token_type: Bearer`,
`expires_in: 3600`. Ein Refresh-Token gibt es in diesem Flow nicht (es wird nur beim Password- und beim
Authorization-Code-Flow ausgestellt); nach einer Stunde wird also ein neues geholt.

Zwei Eigenheiten betreffen einen Adapter direkt [doc]:

* Neue Anwendungen erhalten **JWT**-Tokens (RS256), aus der Version 2.6 migrierte **Opaque**. Das Ausstellen
  eines neuen JWT **widerruft das vorherige**; zwei Prozesse mit demselben Zugang stören sich also, sobald
  beide erneuern. Die Dokumentation nennt dafür den **Device-Scope** (`device_<beliebig>`): damit bekommt jede
  Instanz ein eigenes Token.
* Die Gültigkeitsdauer ist auf Anwendungsebene einstellbar; die Vorgabe sind 3600 Sekunden.

Die Metadatenkataloge brauchen überhaupt kein Token [live], ein Adapter kann einer Person also sagen, *was* er
laden würde und *wie viel*, bevor es Zugangsdaten gibt.

### 4.1 Das API-Portal

`https://apimanager.lantmateriet.se/devportal/apis` ist **ohne Anmeldung** lesbar [live] und listet 15 APIs mit
Version, Kontextpfad und Kontingent; laut Dokumentation kann die Anmeldung nötig sein, um *alle* zu sehen
(„inloggning kan krävas för att lista alla“) [doc], die 15 sind also eine Untergrenze. Sieben davon sind
Atom-Download-Feeds — `/byggnad/atom`, `/marktacke/atom`, `/administrativindelning/atom`,
`/belagenhetsadress/atom`, `/fastighetsomrade/atom`, `/hydrografi/atom` und `/ortnamn/atom` —, dazu
`OGC-Features` (`/ogc-features`), die vier STAC-Kataloge, `FAPI`, `Tumnageldirekt` und ein Demoendpunkt `Hello`
[live]. Die Atom-Feeds sind genau die Schnittstelle, deren Zip-Adressen in den STAC-Objekten stehen (§3.2).

Die Kontingente stehen offen dabei:

| API | Plan | Grenze | Abonnements |
|---|---|---|---|
| `Marktacke_Inspire_Nedladdning` | OpenData | 6000 Requests/min | 405 |
| `OGC-Features` | FairUse | 240 Requests/min | 759 |
| `STAC-vektor` | FairUse | 240 Requests/min | 838 |

Alle Angaben [live]; die Dokumentation sagt es selbst, der Business Plan sei die Policy, die „styr hur många
API-anrop per minut som tillåts“ [doc]. Das ist der Grund, die berichtete Kachelgrenze (§2) **nicht** als
API-Kontingent zu lesen:
240 Anfragen je Minute sind großzügig, und eine Grenze „wenige Kacheln je Vorgang“ steht dort nirgends. Sie
sitzt also eher in der Weboberfläche von Geotorget oder in einer Abo-Bedingung, die erst nach der Anmeldung
sichtbar wird (§10).

**Zwei Drosseln, nicht eine** [doc]: Der Business Plan hängt am Abonnement, dazu kommt die **Per-Token-Quota der
Anwendung** — im WSO2-Portal etwa `10PerMin`, wenn man nichts anderes wählt („Per Token Quota“ in der
Dokumentation von WSO2). Lantmäteriets Anleitung sagt deshalb ausdrücklich, man solle beim Anlegen
`Unlimited` als „Shared Quota for Application Tokens“ wählen; sonst begrenzt die Anwendung selbst, unabhängig
vom Tarif, und die wirksame Grenze ist das Minimum aus beidem.

Auch die Schnittstelle des Portals ist schlüsselfrei: `GET /api/am/devportal/v3/apis?limit=30` liefert alle 15
APIs als JSON [live] — brauchbar, um die Liste zu prüfen, ohne die Seite zu lesen.

**`OGC-Features` ist der interessante Eintrag.** Es ist eine OGC-API-Features-Schnittstelle unter
`https://api.lantmateriet.se/ogc-features/v1` (OpenAPI 3.0.3, 65 Pfade) [live] mit den Produkten `marktacke`
(Collections `markytor` und `sankmarksytor`), `administrativ-indelning` (kommuner, län, rike, je Jahr),
`fastighetsindelning` und `hydrografi`; **Gebäude fehlen**. Antworten sind `application/geo+json`, die
Operationen nehmen `bbox`, `limit`, `offset` und einen CQL2-Filter [live]. Die Collection-Metadaten antworten
ohne Zugangsdaten (`/marktacke/collections` → `200`), die `items` nicht (`401`, „Missing Credentials“, die
Meldung nennt `Authorization : Bearer …`) [live]. Das OpenAPI-Dokument führt zwei Sicherheitsschemata,
`default` und `basic_auth` [live]; Basic statt Bearer wäre für den Fetcher der kleinere Eingriff, weil ein
Header genügt.

### 4.2 Die Verifikationsumgebung

Lantmäteriet betreibt dasselbe Portal auch als Testumgebung, `https://apimanager-ver.lantmateriet.se/devportal/`,
und auch die ist ohne Anmeldung lesbar [live]. Sie listet **17** APIs: die der Produktion ohne `STAC-karta`,
dafür `BRAPI` (`/brapi`) und je eine ältere Version von Administrativ indelning und Belägenhetsadress [live].
Die Kataloge antworten schlüsselfrei (`api-ver.lantmateriet.se`, 61 statt 737 Orthofoto-Sammlungen) [live].

Für einen Adapter taugt sie nicht als Datenquelle: die Assets der Objekte zeigen auf **interne** Hosts über
`http`, etwa `http://download-app00655-pl-stac-boh-ver.internal.caas4.lm.se/hojd/data/grid/mhm/61_3/m619_35.tif`
— dieser Name löst öffentlich nicht auf [live], und `UrlPolicy` würde die Adresse ohnehin ablehnen (kein
`https`, kein öffentlicher Name, §8.1). Ihr Nutzen ist ein anderer: wer Testzugänge bekommt, kann Formate und
Fehlerfälle prüfen, ohne die Produktion zu berühren — und die 15 APIs der Produktion sind, wie die 17 hier
zeigen, nicht die ganze Schnittstellenlandschaft.

---

## 5. CRS: SWEREF99 TM und RH2000

EPSG:3006 (SWEREF99 TM) und EPSG:25833 (ETRS89 / UTM 33N) sind die **gleiche Projektion**: Transverse Mercator,
Ellipsoid GRS 80, Ursprungsbreite 0°, Mittelmeridian 15° O, Maßstabsfaktor 0,9996, falscher Rechtswert 500 000,
falscher Hochwert 0. Ein Koordinatentripel aus 3006 lässt sich daher ohne Rechnung als 25833 verwenden; die
Datums unterscheiden sich um Zentimeter (SWEREF99 ist die nationale Realisierung von ETRS89), was keine
Orientierungskarte sieht. Das zusammengesetzte CRS der Höhen ist EPSG:5845 = 3006 + RH2000, und das nennt
`proj:code` an jedem Höhenobjekt [live].

Die Maßstabsverzerrung ist die von UTM 33 und damit aus Brandenburg und Berlin schon vertraut. Punktmaßstab der
Projektion (gerechnet, GRS 80, `k0` 0,9996, 15° O):

| Ort | Breite | Länge | k | Abweichung |
|---|---|---|---|---|
| Malmö | 55,60 | 13,00 | 0,999795 | −0,2 m/km |
| Simrishamn | 55,56 | 14,35 | 0,999621 | −0,4 m/km |
| Göteborg | 57,71 | 11,97 | 1,000000 | ±0,0 m/km |
| Visby | 57,64 | 18,30 | 1,000076 | +0,1 m/km |
| Stockholm | 59,33 | 18,07 | 0,999974 | −0,03 m/km |
| Östersund | 63,18 | 14,65 | 0,999604 | −0,4 m/km |
| Luleå | 65,58 | 22,15 | 1,000928 | +0,9 m/km |
| Haparanda | 65,83 | 24,13 | 1,001719 | +1,7 m/km |
| Treriksröset | 69,06 | 20,55 | 1,000198 | +0,2 m/km |

An der finnischen Grenze erreicht die Verzerrung +0,17 %, 1,7 m je Kilometer — auf Papier bei 1:4000 sind das
0,43 mm je Kilometer, für eine Startkarte also unsichtbar. freemapper misst durchweg in seinem Karten-CRS, die
Verzerrung gerät also nie in eine Vorlage oder einen Kandidaten.

**Was ein Adapter trotzdem prüfen muss.** Nicht jedes schwedische Produkt liegt in 3006: ein Produkt kann in
einer lokalen SWEREF99-Zone kommen (12 00 … 23 15), und EPSG:5845 darf in einem URL-Parameter nicht für 3006
gehalten werden. Da das `crs` des Manifests nur 25832 und 25833 kennt, würde ein schwedisches Manifest
`crs = 25833` schreiben, und der Code übersetzte das `3006`/`5845` des Assets darauf und lehnte alles andere
ab — genau wie die deutschen Adapter 25832/25833 in WMS-Anfragen übersetzen.

Die Höhen sind RH2000. freemapper liest z-Werte, wie sie kommen, und legt sie in die Höhenlinien des
Geländemodells; solange jede schwedische Quelle RH2000 benutzt, ist keine Umrechnung nötig, und ein schwedisches
Projekt mischt sie nie mit DHHN2016 einer deutschen Quelle.

---

## 6. OSM

* `https://download.geofabrik.de/europe/sweden-latest.osm.pbf` → `200`, **982 465 405 B**, Last-Modified
  2026-11-16 03:24 [live]. `sweden.html` sagt „No sub extracts“ [live], es gibt also keinen Schnitt nach
  Provinz oder Gemeinde; jedes Projekt in Schweden läse das Ganzland-Extrakt. `sweden-updates/`, `.md5` und
  `.poly` gibt es [live].
* `MAX_EXTRACT_BYTES` in `crates/freemapper/src/sources/osm.rs` ist 2 GiB, und das Extrakt wird 7 Tage gehalten
  (`MAX_EXTRACT_AGE_DAYS`); 982 MB sind also erlaubt, aber mit Abstand das größte Extrakt, das eine Region
  verlangt (Brandenburg: angekündigt 300 MB; Niedersachsen: 510 MB).
* Die OSM-Relation Schwedens ist `52822`, `boundary=administrative`, country, bbox 55,137–69,060° N,
  10,594–24,178° O [live, Nominatim]; `freemapper init --place` und die Flächenseite können sie nutzen wie in
  Deutschland. `download.geofabrik.de` steht in `CORE_HOSTS`, OSM funktioniert also für jedes Land ohne Manifest.

---

## 7. Schlüsselfreie Quellen und Dokumentation

### 7.1 Naturvårdsverket: Schutzgebiete

* `https://geodata.naturvardsverket.se/geoserver/wms` → `200`, `Fees: NONE`, `AccessConstraints: NONE`, 87
  benannte Layer [live]. GetMap in **EPSG:3006** funktioniert: Layer `ps-nvr:PS.ProtectedSites.NR` für ein
  10-km-Quadrat antwortete `200 image/png` in 10 183 B [live].
* Layernamen (die WMS-Styles tragen andere Namen — `<Layer><Name>` auswerten): `ps-nvr:PS.ProtectedSites.NR`
  (naturreservat), `.NP` (nationalparker), `.NM`, `.KR` (kulturreservat), `.OBO`, `.DVO`, `.NVO`, `.IF`;
  `ps-n2k:PS.ProtectedSites.Natura2000`; `ps-ic:PS.ProtectedSites.Bio`, `.Helcom`;
  `am-restriction:AM.drinkingWaterProtectionArea`; Bodenbedeckung `lc-nmd:LC.LandCoverRaster.Bas.2023.v2.x`
  [live].
* **Der WFS ist abgeschaltet**: `…/geoserver/wfs?service=WFS&request=GetCapabilities` antwortet `200` mit einem
  `ExceptionReport` — „Service WFS is disabled“ [live]. Ein Statuscode allein beweist hier keinen Dienst.
* Metadaten: der Katalog `https://geodatakatalogen.naturvardsverket.se/srv/eng/csw` (CSW 2.0.2, ISO 19139)
  antwortete [live]; seine Datensätze zu den Schutzgebieten sagen `Öppna data`, `Värdefulla datamängder`,
  `Inspire` und als Einschränkung `[RESTRIKTIONER]=Creative commons CC0 1.0 Universiell`, mit Verweisen auf ein
  WMS 1.3.0 und einen Shapefile-Download in `EPSG:3006` [live].
* Das ist die eine schwedische Quelle, die freemapper heute ohne Zugangsdaten nutzen könnte, und die, die ein
  Erlaubnishinweis am meisten braucht: Naturreservate und Natura-2000-Gebiete sind die Stellen, an denen ein
  Orientierungslauf die Erlaubnis der Provinzverwaltung braucht.

### 7.2 Nationella marktäckedata: Bodenbedeckung ohne Konto

Naturvårdsverket führt mit zehn anderen Behörden (darunter Lantmäteriet und Skogsstyrelsen) eine landesweite
Bodenbedeckungskarte, die **ohne Konto** abrufbar ist — für freemapper die einzige schlüsselfreie Quelle, die an
die Stelle von `marktacke` treten könnte:

* **Basisschicht**: 54 thematische Klassen in vier Hierarchiestufen (NMD2023 v2.x), Raster mit **10 m**, CRS
  SWEREF99 TM (EPSG:3006), als **16-Bit-Ganzzahl** und GeoTIFF mit **PackBits** geliefert; ganz Schweden als
  Zip mit 2 705 971 730 B [live], die ältere NMD2018 (25 Klassen, 1,4 GB) daneben [live]. Je Provinz einzeln
  im offenen Verzeichnisindex `…/NMD2018/bas_lan_ogen/` (Stockholm 33 MB) [live] — der Bezug ist also
  stückelbar, anders als bei Lantmäteriet (§10).
* **Zusatzschichten**: *objekthöjd och objekttäckning* mit vier 10-m-Rastern (Objekthöhen 0,5–5 m und 5–45 m,
  dazu der Bedeckungsgrad beider Höhenbänder), ferner Produktivität, Landnutzung, Baumarten, Gebirgswald und
  Bodenbearbeitung [live] [doc]. Das Höhenband ist ein grober nDSM-Ersatz für die Baumlogik der Trainingskarte
  — auf 10 m, also ohne Einzelbäume.
* **Lizenz**: die Policy von Naturvårdsverket sagt „Rapporter, bilder, filmer och ljudklipp ska licensieras med
  CC-by licens. Övriga data ska licensieras med CC0.“ und „Naturvårdsverkets data som tillgängliggörs med
  CC0 är fria att använda, sprida, göra om, modifiera och bygga vidare på“ [doc] — also CC0 1.0 ohne
  Namensnennungspflicht (die führen wir trotzdem).
* **Darstellung**: ein WMS unter `…/inspire/lc-nmd/ows` mit EPSG:3006 und den Layern der Jahre 2018 und 2023
  antwortet schlüsselfrei [live].
* **Vorbehalt**: NMD2023 ist noch in Produktion; 2026 liegt der Schwerpunkt auf den Fjällen [doc]. Ob jede
  Kartenfläche schon in v2.x vorliegt, muss ein Adapter je Fläche prüfen, und die 10 m sind gröber als
  Lantmäteriets Vektordaten.

### 7.3 Copernicus DEM

`https://copernicus-dem-30m.s3.amazonaws.com/…/Copernicus_DSM_COG_10_N55_00_E013_00_DEM.tif` antwortete `206`
auf eine Range-Anfrage [live] — ein schlüsselfreier, grober (30 m) Gelände-Rückfall zu den Bedingungen des
Copernicus DEM (freie Nutzung mit Namensnennung). Zu grob für Höhenlinien im 5-m-Abstand, brauchbar als Prüfung.

### 7.4 Was nicht antwortet

| Dienst | Antwort [live] |
|---|---|
| Skogsstyrelsen WFS (`geodata.skogsstyrelsen.se/geoserver/wfs`) | `403` |
| Lantmäteriet Topowebb WMTS (`api.lantmateriet.se/open/topowebb-ccby/v1/wmts/…`) | `503` |
| Lantmäteriet Topowebb WMS (`…/open/topowebb-ccby/v1/wms`) | `404` |
| `maps.lantmateriet.se/topowebb/wms` | `404` |
| Geotorget-Startseite | `200`, aber der Produktkatalog liegt hinter der Anmeldung |

Die Topowebb-Endpunkte waren einmal offene Daten mit Schlüssel; die `503` deutet darauf, dass sie für anonyme
Nutzung geschlossen sind [unverified, ob das dauerhaft ist].

### 7.5 HMK: die Handbücher

Die Handbuchreihe HMK (Handbok i mät- och kartfrågor) liegt als offene PDFs auf `lantmateriet.se` [live], ohne
Konto: *Höjdmodell 2023*, *Ortofoto 2017*, *Flygburen laserskanning 2024*, *Grundkarta 2025*, *Kartografi
2025*, *Fotogrammetrisk detaljmätning 2026* und weitere, dazu vier gemeinsame Referenzdokumente
(*Introduktion*, *Geodatakvalitet*, *Geodetisk infrastruktur*, *Ordlista*). Sie sind keine Datenquelle, sondern
die Produktionsvorgaben — und damit die `[doc]`-Grundlage für das, was ein Adapter von schwedischen Daten
erwarten darf:

* **Höhenmodelle**: als Raster normalerweise GeoTIFF, geografisch in einem Gitternetz geteilt, dessen
  Dateinamen die Koordinaten der **Südwest-Ecke** enthalten, nach Lantmäteriets SWEREF-99-Indexsystem; jede
  Kompression muss verlustfrei sein (HMK – Höjdmodell 2023, Kap. 2.4). Das erklärt die beobachtete Benennung
  `m617_38` und die Deflate-Kacheln aus §3.5.
* **Orthofotos**: Bildtyp PAN, RGB oder CIR (NIR/Rot/Grün), üblich sind **8 Bit je Kanal** (24-Bit-RGB), und
  für die Abgabe wird unkomprimiertes TIFF/GeoTIFF empfohlen (HMK – Ortofoto 2017, Kap. 2.3.5 und 2.4.2). Das
  `RGBI` der Landesprodukte [live] ist also der Standard, nicht eine Eigenheit der Kacheln.
* **Zugang**: HMK – Introduktion 2017 zitiert auf S. 24 die Geodatengesetzgebung (SFS 2010:1767 und 2010:1770):
  informationsansvariga Behörden müssen Geodaten und Geodatendienste für alle zugänglich machen, die
  Öffentlichkeit soll Geodaten über das Internet **suchen, ansehen und herunterladen** können, und Suchen und
  Ansehen über Dienste sollen kostenfrei sein [doc]. Der Rahmen verlangt also Zugang; die Umsetzung verlangt
  ein Konto (§2) und begrenzt den Bezug (§10). Ob die berichtete Kachelgrenze mit dieser Pflicht im Verhältnis
  steht, ist eine Rechtsfrage und hier nicht beantwortet.

### 7.6 Geodataportalen: ein schlüsselfreier Katalog

`https://www.geodata.se/geodataportalen/srv/eng/csw` ist ein CSW 2.0.2 der schwedischen Geodateninfrastruktur
und antwortet ohne Konto [live]. Er liefert keine Daten, ist aber der Weg, auf dem die schlüsselfreien Quellen
auffindbar sind: die Datensätze zu NMD, zu SGU und zu den Länsstyrelsen tragen dort ihre Download- und
WMS-Adressen [live] — die NMD-Verweise in §7.2 stammen von dort.

Zwei Eigenheiten für den, der ihn abfragt [live]:

* `GetRecords` braucht `CONSTRAINT_LANGUAGE_VERSION=1.1.0`, sonst kommt `MissingParameterValue`.
* Ein `%` in der CQL-Bedingung lässt den Dienst mit `UnknownFormatConversionException` abbrechen, ein
  Platzhalter ist also nicht zu gebrauchen. Mehrwortige und schwedische Suchbegriffe treffen schlecht
  (`AnyText like 'Marktäckedata'` → 0, `'grunddata'` → 19); einzelne Wortstämme treffen.

### 7.7 Demodata: Höhen ohne Konto

Lantmäteriet stellt zwei Demogebiete bereit — Göteborg Majorna (Stadt) und Emån (Wald) —, „avgiftsfritt“ für
„test och utvärdering“; wer sie kommerziell nutzt oder weitergibt, braucht einen Vertrag [live]. Die Dateien
liegen unter `https://www.lantmateriet.se/geodata_data/demodata_testdata/GEO-demoomrade/`.

* **Oberflächenmodell**: `…/Majorna/Hojddata/Ythojdmodell_flygbilder/M_ythojdmodell_tif.zip` mit 352 615 030 B,
  für Emån 349 235 175 B [live]. Im Zip liegt `63_3/639_31_7550_2020_dsm.tif` mit **10 000 × 10 000 px,
  Float32, 0,25 m, EPSG:3006, Nodata −9999, LZW mit Prädiktor 2, Kacheln 256 × 256**, daneben
  `…_class.tif` (8 Bit, LZW, Nodata 255), ein `saknade_ytor`-RGB-Bild mit 2 m Auflösung und JSON-Sidecars
  [live].
* **Laserdata NH** und **Digitala flygbilder** liegen ebenso als Demo vor [live]; dieselben Dateien gibt es
  laut Seite auch per FTP, `ftp://download-open.lantmateriet.se/Geodata/demodata_testdata/` [doc]. Ein
  **Orthofoto-Demo gibt es nicht**; die Kompression der Ortofoto-COGs bleibt damit offen (§10).
* **Folge für §8.4**: Das Demomodell liest `tiff_range` heute schon — ein Float32-Sample, LZW, Kacheln,
  Geotags, Nodata. Die Produktion dagegen ist **deflate** (§3.4). Die Lücke ist also die Kompression der
  Produktion, nicht das Format, und sie lässt sich am Demo nicht reproduzieren.

---

## 8. Was freemapper bräuchte

Jeder Punkt unten wurde am 2026-11-16 gegen den Code geprüft; Datei- und Symbolnamen stammen aus diesem
Repository.

### 8.1 Ein Token im Fetcher

`Fetcher::get(&str)` und `get_range(&str, ByteRange)` (`crates/freemapper-adapter-api/src/fetch.rs`) nehmen
eine URL und sonst nichts; der HTTP-Fetcher setzt `User-Agent`, `Range` und `Accept-Encoding` und keinen weiteren
Header (`crates/freemapper/src/fetch.rs`). Es gibt keinen Platz für ein Bearer-Token, keinen Token-Endpunkt,
keine Ablaufbehandlung, und `UrlPolicy` entscheidet nur über Hosts, nicht über Zugangsdaten. Ein schwedischer
Adapter braucht einen Token-Provider in der Adapter-API und die Regel, dass das Token nie in `freemapper.toml`,
in Cache-Dateinamen, in Fortschrittszeilen oder in `ATTRIBUTION.txt` gelangt.

### 8.2 Ein GeoPackage- und ein OAPIF-Leser

`byggnader`, `marktacke`, `kommun-lan-rike` und das `breakgeometry`-Asset sind GeoPackage, also SQLite mit WKB
in einer Tabelle. Der Workspace hat überhaupt keine SQLite-Abhängigkeit (`grep sqlite|rusqlite|geopackage` in
jeder `Cargo.toml` findet nichts) und keinen GeoPackage-Leser.

**Aber es muss nicht alles GeoPackage sein.** Die OGC-Features-API (§4.1) liefert `marktacke`
(`markytor`, `sankmarksytor`), `administrativ-indelning` (kommuner, län, rike), `fastighetsindelning` und
`hydrografi` als **GeoJSON** (`application/geo+json`, `bbox`, `limit`, `offset`, CQL2-Filter). freemapper hat
mit `geojson_wfs` (`crates/freemapper-sources/src/geojson_wfs.rs`) bereits einen Leser für
GeoJSON-FeatureCollections samt Paging — er fragt WFS 2.0 mit `typeNames`/`count`/`startIndex`, ein
OAPIF-Geschwister braucht Pfad-Collections und `limit`/`offset`, aber keinen SQLite-Stack. GeoPackage bleibt
für die **Gebäude** und das `breakgeometry`-Asset nötig, samt Zip drumherum.

### 8.3 Ein COG-Leser für das Orthofoto

`tiff_range` (`crates/freemapper-sources/src/tiff_range.rs`, ADR 0141) liest **einen Float32-Sample, LZW
komprimiert, Prädiktor 1–3**, mit `ModelPixelScale` (33550) und `ModelTiepoint` (33922), in Blöcken von
höchstens 128 MB. Das schwedische Orthofoto ist 8-Bit-RGBI in einem COG; der Leser lehnt es bei seiner ersten
Prüfung ab (`samples, bits, format != (1, 32, 3)`). Die 8 Bit je Kanal sind der schwedische Standard (HMK –
Ortofoto 2017, §7.5), kein Ausreißer einzelner Kacheln. Die Orthofoto-Vorlage bräuchte stattdessen einen
gekachelten 8-Bit-Leser mit Bandverschachtelung und entweder einem JPEG- oder einem deflate-Dekoder. Ohne ihn
gibt es in Schweden gar kein Orthofoto — kein WMS liefert eines, und ein Kachel-Download von 0,6 GB je 2,5 km
ist die einzige Alternative.

### 8.4 Deflate und PackBits im Rasterleser

Die COGs von DTM und DSM sind in Produktion **deflate** (Kompression 8) und 512 × 512 gekachelt [live]; das
Demomodell aus §7.7 ist dagegen LZW und damit heute schon lesbar. `tiff_range` verlangt
Kompression **5** und lehnt `8` rundheraus ab. Mit NMD (§7.2) kommt eine dritte Kompression hinzu: **PackBits**
(32773), dazu 16-Bit-Ganzzahlen statt Float32. Das ist die kleinste der Lücken: zwei weitere Dekoder neben
`decode_lzw_block`, dazu je eine Testdatei. Damit wird der DTM per Range lesbar wie ein deutsches DOM1 — die
10-km-Kachel kostet dann nur die Blöcke, die die Kartenfläche braucht — und NMD lesbar ohne den 2,7-GB-Umweg.

### 8.5 Das 2,5-km-Gitter gegen `RasterKind::Tiles`

`RasterKind::Tiles { grid_km: u32 }` (`crates/freemapper-adapter-api/src/rasters.rs`) heißt *Kacheln eines
ganzen Kilometerbetrags in EPSG:25833, benannt nach ihrer Südwest-Ecke in km*. Der DTM passt: 10-km-Kacheln auf
Vielfachen von 10 000 m. DSM und Orthofoto nicht: ihre Kacheln sind 2,5 km auf Vielfachen von 2 500 m, was
weder ein 2-km- noch ein 3-km-Gitter abbildet. Entweder näht das Backend 2,5-km-Kacheln selbst in ein
Kilometergitter, oder `RasterKind` bekommt ein Gitter in Metern. Dasselbe gilt für das DSM mit 0,25 m, während
die deutschen DOMs 0,2 m und 1 m haben — die Vorlagen entstehen in fester Pixelgröße (`MAX_RASTER_SIDE_PX`
20 000 bei 0,2 m ⇒ höchstens 4 km je Seite), eine 0,25-m-Quelle wird also heruntergerechnet, was in Ordnung
ist, aber ein Adapter darf es nicht voraussetzen.

### 8.6 „Keine Raster, keine Karte“

`RegionAdapter::rasters` dokumentiert `None` als „the region has ALKIS but no rasters, and no map can have its
centre in it“ (`crates/freemapper-adapter-api/src/adapter.rs`), und `check_area` lehnt eine Fläche ab, wenn
`raster_region_in` keinen Adapter findet (`crates/freemapper/src/project.rs`, `ProjectError::OutsideCoverage`).
Eine schwedische Region allein auf OSM **kann also keine Karte beherbergen**, so viel OSM sie auch hat. Das
NMD-Raster aus §7.2 hilft hier nicht: `RasterRecords` (`crates/freemapper-adapter-api/src/project.rs`) kennt
Kacheln, Orthofoto, Gelände und Oberfläche, keine Bodenbedeckung, und die Regel hängt an diesen dreien.
Schweden als „OSM und NMD“ aufzunehmen braucht daher eine bewusste Entscheidung, diese Regel für eine Region
ohne solche Raster zu lockern — ein ADR, kein Manifest.

### 8.7 Die Grenze

`boundary.geojson` in EPSG:25833 (oder 25832) wird für jeden deutschen Adapter von Hand gebaut. Für Schweden
ist die natürliche Quelle `kommun-lan-rike_aktuell.zip`, ein 6-MB-GeoPackage mit allen drei Ebenen [live]; die
Grenze würde also einmal von Hand daraus geschnitten und eingecheckt, oder der Adapter läse das GeoPackage.

### 8.8 Namensnennung, DSGVO, Erlaubnishinweise

| Punkt | Was ein schwedischer Adapter schriebe |
|---|---|
| Namensnennung | CC BY 4.0 für Höhen, Vektoren und Orthofoto, Quellenangabe „© Lantmäteriet“ — die Lizenz nennt sich selbst, wo die deutsche dl-de/by-2-0 einen festen Hinweis vorschreibt |
| Orthofoto und Gebäude | Die Startseite des `bild`-Katalogs sagt „Din användning kommer att prövas juridiskt, i enlighet med GDPR, och du behöver godkänna särskilda användningsvillkor“ [live]. Ein Hinweis an die Nutzerin müsste sagen, dass das geladene Orthofoto Menschen und Fahrzeuge zeigen kann und dass das Veröffentlichen der Karte ihre eigene DSGVO-Entscheidung ist |
| Erlaubnishinweise | Der WMS/CC0-Datensatz von Naturvårdsverket für Reservate und Natura 2000 ist die Schutzgebietsquelle. Betreiber eines Waldes stehen in keinem offenen schwedischen Datensatz: der WFS von Skogsstyrelsen ist `403` [live], und die Grundstückseigentümer stehen in `fastighetsindelning`, also in personenbezogenen Daten. Allemansrätten (das Recht auf öffentlichen Zugang) lässt jeden im Gelände gehen und laufen, eine organisierte Veranstaltung braucht aber die Zustimmung des Grundeigentümers [doc]. Ein schwedisches `permits.md` müsste das deutlich sagen |

---

## 9. Aufwand und Optionen

Grobe Größen, in den Einheiten, in denen die deutschen Adapter gebaut wurden; alle drei sind Schätzungen, keine
Messungen.

| Option | Arbeit | Ergebnis |
|---|---|---|
| **A. Voller Adapter** | Token-Provider in der Adapter-API und im Fetcher; deflate- (und 8-Bit-)TIFF-Range-Leser; OAPIF-Leser (GeoJSON) und GeoPackage-Leser für Gebäude; `RasterKind`-Gitter in Metern oder ein nähendes Backend; Grenze; Manifest; Namensnennung; Tests mit aufgezeichneten Fixtures; dazu ein Weg für den Bezug, wenn die berichtete Kachelgrenze auch für die API gilt (§10). Etwa 6 bis 11 fokussierte Aufgaben, das erste freemapper-Feature, das Zugangsdaten halten muss — und das erste, für das eine Nutzerin bezahlt (§1). | Eine schwedische Karte wie eine deutsche: Gebäude, Bodenbedeckung, DTM-Höhenlinien, Orthofoto, nDSM aus Daten mit `other`-Lizenz und unklarer Rechtsgrundlage. |
| **B. OSM, NMD und Schutzgebiete** | Ein ADR, das eine Region zulässt, deren `rasters()` `None` ist; eine Region mit Grenze und ohne Raster-Backend; OSM als Vektoren; NMD als Bodenbedeckung und grober Objekthöhe (§7.2); Schutzgebiete; ein Leser für 16-Bit-PackBits-TIFF. Keine Zugangsdaten. | Eine Karte mit OSM-Wegen und -Gebäuden, Bodenbedeckung und Baumdichte aus NMD auf 10 m, Schutzgebieten; keine Höhenlinien, kein Orthofoto, keine Einzelbäume. |
| **C. Nur Dokumentation** | Diese Datei. | Eine Antwort für die nächste Person, die fragt, einschließlich der sechs Tatsachen, die entscheiden: Token, Kachelgrenze, GeoPackage, deflate-COG, NMD als schlüsselfreie Bodenbedeckung, „keine Raster, keine Karte“. |

B ist billig, erzeugt aber eine Karte, die sichtbar ärmer ist als eine deutsche, und braucht ein ADR, das eine
Regel für alle Regionen ändert. A ist die ehrliche Fassung und hängt an den Zugangsdaten und den beiden Lesern.
C ist das, was diese Änderung liefert.

Offene Fragen, bevor A beginnt:

1. Darf freemapper überhaupt fremde Zugangsdaten halten? Wo lägen sie (Schlüsselbund, Umgebungsvariable,
   Konfigurationsdatei), und wie kommt ein kopfloser MCP-Server daran?
2. Ist `other` eine annehmbare Lizenz für das Oberflächenmodell und die Laserdaten, oder muss A ohne nDSM
   auskommen? Ohne eines hätte die Trainingskarte in Schweden keine Bäume und keine Unterscheidung von Park und
   Wald.
3. Ist das Orthofoto einen neuen 8-Bit-COG-Leser wert, angesichts der DSGVO-Prüfung und der 0,6-GB-Kacheln?
4. Bekommt `RasterKind` ein Gitter in Metern, oder näht jeder Anbieter außerhalb des Kilometergitters selbst?
5. Gilt die berichtete Grenze von wenigen Kacheln je Vorgang nur im Web-Download von Geotorget, oder auch für
   die API? Im zweiten Fall braucht der Bezug einer Karte einen Menschen oder eine Warteschlange — siehe §10.

---

## 10. Nicht geprüft

* Die Token-Anfrage mit echten Client-Zugangsdaten: Endpunkt, Form und Lebensdauer stehen in der
  Portal-Dokumentation (§4), ein Aufruf mit gültigen Daten ist aber nicht geprüft, und ob `dl1` ein Token für
  jedes Produkt annimmt, erst recht nicht — es wurde kein Konto angelegt. Das ist der eine Teil, den eine
  Person mit Zugang vor dem Start von A bestätigen muss.
* Ob die Grenze von wenigen Kacheln je Vorgang, die der schwedische Orientierungsläufer berichtet, für die
  Weboberfläche von Geotorget gilt oder auch für die API-Assets unter `dl1.lantmateriet.se`. Das API-Portal
  nennt 240 bis 6000 Requests je Minute und keine Kachelgrenze (§4.1), spricht also für die Weboberfläche oder
  für eine Abo-Bedingung; ob zusätzlich eine Tages- oder Volumengrenze auf den Assets liegt, bliebe zu prüfen.
  Gilt sie auch für die API, kann kein Adapter eine Karte unbeaufsichtigt laden.
* Ob die `other`-Lizenz von `dsm-cog`, `dsm-skoglig-copc` und `orto-dsm-cog` abgeleitete Produkte in einer
  veröffentlichten Karte erlaubt. Die Sammlung sagt nur `other`; die Bedingungen liegen hinter Geotorget.
* Ob die Orthofoto-COGs intern JPEG- oder deflate-komprimiert sind — das `info`-Asset einer Orthofoto-Kachel
  wurde nicht geholt (seine URL braucht ein Token, anders als die Höhen-`info`-Assets unter `/hojd/pub/`).
* Ob jede Orthofoto-Sammlung 0,16 m hat; drei Stichproben in zwei Gebieten hatten es.
* Ob RH2000 und DHHN2016 sich um eine Konstante unterscheiden, die zählte, wenn eine Karte je beide mischte.
  Für ein schwedisches Projekt allein nicht.
* Ob NMD2023 v2.x schon jede Kartenfläche abdeckt; die Produktion läuft, 2026 mit Schwerpunkt auf den
  Fjällen [doc].
* Ob CC0 auch für die Zusatzschichten des NMD gilt; die Policy spricht von Daten allgemein, die Datensätze
  selbst tragen keinen Lizenztext [doc].
* Ob die Ortofoto-COGs als Demo vorliegen — die Demodata-Seite listet nur die digitalen Flugbilder [live];
  damit bleibt ihre Kompression ungeprüft (PackBits? JPEG? deflate?).
* Was die kostenpflichtigen Produkte kosten; die Übersicht nennt nur „Avgift“, keine Preise [live].
* Ob die `503` des Topowebb-WMTS dauerhaft ist.
* Die genaue Lizenz von `marktacke` je Feature-Klasse; die Sammlung sagt CC BY 4.0.

---

## 11. Protokoll der Abfragen

Befehle, gekürzt; jede Zeile wurde am 2026-11-16 ausgeführt.

```sh
# STAC, schlüsselfrei
curl -s https://api.lantmateriet.se/stac-hojd/v1/collections        # 3 Sammlungen
curl -s https://api.lantmateriet.se/stac-vektor/v1/collections      # 6 Sammlungen, alle CC-BY-4.0
curl -s https://api.lantmateriet.se/stac-bild/v1/collections        # 737 Sammlungen
curl -s https://api.lantmateriet.se/stac-hojd/v1/conformance        # item-search#sort/#fields/#query, CQL2
curl -s https://api.lantmateriet.se/stac-hojd/v1/api                # OpenAPI 3.1.0, keine securitySchemes
curl -s https://api.lantmateriet.se/stac-hojd/v1/collections/dtm-cog/items/617_38
curl -s https://dl1.lantmateriet.se/hojd/pub/grid/mhm/61_3/m617_38_info.json
curl -s https://api.lantmateriet.se/stac-hojd/v1/collections/dsm-cog/items/2026_a2-617_39_5000
curl -s 'https://api.lantmateriet.se/stac-hojd/v1/search?collections=dtm-cog&bbox=13.10,55.68,13.25,55.75&limit=3'
curl -s https://api.lantmateriet.se/stac-bild/v1/collections/orto-a2-2026/items/o61775_3825_25_mr26
curl -s https://api.lantmateriet.se/stac-vektor/v1/collections/byggnader/items/1281
curl -s https://api.lantmateriet.se/stac-vektor/v1/collections/kommun-lan-rike/items/aktuell

# Authentifizierung
curl -s -o /dev/null -D - https://dl1.lantmateriet.se/hojd/data/grid/mhm/61_3/m617_38.tif   # 401
curl -s -o /dev/null -D - -H 'Authorization: Bearer abc' …/m617_38.tif                     # 401 invalid_token
curl -s -X POST https://api.lantmateriet.se/token -d grant_type=client_credentials          # 401 invalid_client
curl -s -X POST https://api.lantmateriet.se/token -u abc:def -d grant_type=client_credentials  # 401
curl -s -X POST https://apimanager.lantmateriet.se/oauth2/token -d grant_type=client_credentials   # 401, gleiche Form
curl -s https://apimanager.lantmateriet.se/docs/                                                  # Doku ohne Anmeldung

# Demodata, ohne Konto (§7.7)
curl -sIL https://www.lantmateriet.se/geodata_data/demodata_testdata/GEO-demoomrade/Majorna/Hojddata/Ythojdmodell_flygbilder/M_ythojdmodell_tif.zip   # 352615030 B
# im Zip: 639_31_7550_2020_dsm.tif 10000x10000 Float32, 0,25 m, LZW Prädiktor 2, Kacheln 256, Nodata -9999

# OSM und schlüsselfreie Quellen
curl -sIL https://download.geofabrik.de/europe/sweden-latest.osm.pbf        # 982465405 B
curl -s https://download.geofabrik.de/europe/sweden.html                    # "No sub extracts"
curl -s 'https://geodata.naturvardsverket.se/geoserver/wms?service=WMS&request=GetCapabilities&version=1.3.0'
curl -s 'https://geodata.naturvardsverket.se/geoserver/wms?service=WMS&version=1.3.0&request=GetMap&layers=ps-nvr:PS.ProtectedSites.NR&crs=EPSG:3006&bbox=380000,6170000,390000,6180000&width=800&height=800&format=image/png'
curl -s 'https://geodata.naturvardsverket.se/geoserver/wfs?service=WFS&request=GetCapabilities'  # Exception
curl -s 'https://geodatakatalogen.naturvardsverket.se/srv/eng/csw?service=CSW&request=GetCapabilities&version=2.0.2'
curl -s -r 0-99 https://copernicus-dem-30m.s3.amazonaws.com/Copernicus_DSM_COG_10_N55_00_E013_00_DEM/Copernicus_DSM_COG_10_N55_00_E013_00_DEM.tif
curl -s -o /dev/null -w '%{http_code}\n' https://geodata.skogsstyrelsen.se/geoserver/wfs?service=WFS&request=GetCapabilities  # 403
curl -s -o /dev/null -w '%{http_code}\n' https://api.lantmateriet.se/open/topowebb-ccby/v1/wmts/1.0.0/WMTSCapabilities.xml   # 503

# Bodenbedeckung ohne Konto: NMD und der Katalog der Geodateninfrastruktur
curl -s 'https://geodata.naturvardsverket.se/inspire/lc-nmd/ows?service=WMS&request=GetCapabilities&version=1.3.0'   # 200, EPSG:3006
curl -sIL https://geodata.naturvardsverket.se/nedladdning/marktacke/NMD2023/Basskikt_v2_x/NMD2023_basskikt_v2_1.zip   # 2705971730 B
curl -sIL https://geodata.naturvardsverket.se/nedladdning/marktacke/NMD2018/NMD2018_basskikt_ogeneraliserad_Sverige_v1_1.zip   # 1406790147 B
curl -s https://geodata.naturvardsverket.se/nedladdning/marktacke/NMD2018/bas_lan_ogen/   # Verzeichnisindex, AB 33 MB
curl -s 'https://www.geodata.se/geodataportalen/srv/eng/csw?service=CSW&request=GetCapabilities&version=2.0.2'   # CSW 2.0.2

# API-Portal und der vierte STAC-Katalog
curl -s 'https://apimanager.lantmateriet.se/api/am/devportal/v3/apis?limit=30'   # 15 APIs, JSON, ohne Token
curl -s https://api.lantmateriet.se/stac-karta/v1/collections                    # topowebb, nmk250, nmk50
curl -s 'https://api.lantmateriet.se/stac-karta/v1/collections/topowebb/items?limit=2'   # gpkg, 175313006592 B
curl -s https://api.lantmateriet.se/ogc-features/v1/marktacke/collections                 # 200, markytor
curl -s 'https://api.lantmateriet.se/ogc-features/v1/administrativ-indelning/collections/kommuner/items?limit=1'   # 401
curl -s https://apimanager.lantmateriet.se/api/am/devportal/v3/apis/8e4e3f62-99f4-4734-8faf-5b3245d3baa8/swagger   # OpenAPI 3.0.3

# Testumgebung
curl -s 'https://apimanager-ver.lantmateriet.se/api/am/devportal/v3/apis?limit=40'   # 17 APIs
curl -s https://api-ver.lantmateriet.se/stac-hojd/v1/collections/dtm-cog/items?limit=2   # Assets auf interne Hosts

# HMK-Handbücher, schlüsselfreie PDFs
curl -sI https://www.lantmateriet.se/globalassets/om-lantmateriet/var-samverkan-med-andra/hmk/handbocker/hmk_hojdmodell_2023.pdf   # 200
curl -sI https://www.lantmateriet.se/globalassets/om-lantmateriet/var-samverkan-med-andra/hmk/handbocker/hmk_ortofoto_2017.pdf     # 200
curl -sI https://www.lantmateriet.se/globalassets/om-lantmateriet/var-samverkan-med-andra/hmk/handbocker/introduktion-2017.pdf    # 200
```

Die Maßstabsfaktoren in §5 wurden mit der üblichen Transverse-Mercator-Reihe (GRS 80, `k0` 0,9996, 15° O)
gerechnet und stimmen mit den veröffentlichten Werten von SWEREF99 TM bis auf die letzte gezeigte Stelle überein.
