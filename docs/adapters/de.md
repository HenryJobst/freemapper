# Eine Region zu freemapper hinzufügen

Eine Anleitung für alle, die freemapper Karten in einem Bundesland oder einer Region zeichnen lassen wollen,
die es noch nicht abdeckt. Englisch: [en.md](en.md). Das [Tutorial](tutorial.de.md) baut einen Adapter Schritt
für Schritt, die [Referenz](manifest-v1.de.md) führt jeden Schlüssel des Manifests auf.

## 1. Für wen das ist und was Sie brauchen

freemapper bekommt die Daten einer Region über einen Regionsadapter. Die deutschen Bundesländer, die es
abdeckt, sind eingebaut. Eine weitere Region ist ein Ordner mit einer Textdatei, `manifest.toml`, die die
Dienste der Region nennt, und einer Grenzdatei. Sind die Dienste Standarddienste, ist das alles, und Sie
schreiben keinen Code.

Sie müssen die Geodatendienste der Region kennen oder bereit sein, ihre Capabilities zu lesen. Klären Sie vor
dem Anfang vier Dinge:

- Eine Lizenz, die es erlaubt. freemapper lädt die Daten herunter, leitet Kartenobjekte daraus ab und
  schreibt Karten, die weitergegeben werden. Die Daten müssen offen sein: dl-de/zero-2-0, dl-de/by-2-0,
  CC BY 4.0, CC0, ODbL oder eine andere Lizenz, deren Text Sie zitieren können. „Kostenlos ansehen“ genügt
  nicht.
- Das Kataster: Gebäude und Nutzung im vereinfachten ALKIS-Schema der AdV („ALKIS vereinfacht“), aus einem
  WFS oder einem Dienst nach OGC API Features.
- Die Raster: ein Luftbild, ein Geländemodell (DGM1) und, wenn es eines gibt, ein Oberflächenmodell (DOM1)
  mit 1 m.
- Eine Grenze der Region als Polygon.

Eine Karte wird in EPSG:25832 oder EPSG:25833 gezeichnet. Die Region muss also dort liegen, wo UTM-Zone 32
oder 33 passt: in Deutschland und bei seinen Nachbarn.

## 2. Entscheiden: nur Manifest, Manifest mit Backend oder Code

Gehen Sie die Fragen der Reihe nach durch.

1. Kommt das Kataster aus einem WFS oder einem Dienst nach OGC API Features mit den Attributen von „ALKIS
   vereinfacht“ (`oid`, `gebnutzbez`, `funktion` bei Gebäuden; `nutzart`, `bez` bei der Nutzung)?
   - Ja: Das Manifest deklariert es. Weiter mit 2.
   - Nein (ein anderes Schema, das Kataster eines anderen Landes, Dateien zum Herunterladen): Die Region
     braucht einen **Code-Adapter** in Rust, der ihre Daten in dieses Schema bringt. Berlin ist einer. Legen
     Sie ein Issue an, bevor Sie anfangen; das ist mehr als ein Manifest.
2. Kommt das Luftbild aus einem WMS oder WCS, und kommt jedes Höhenmodell aus einem WCS 2.0, der ein GeoTIFF
   mit Höhen im 1-m-Raster liefert?
   - Ja: **nur Manifest**. Sachsen-Anhalt ist das Beispiel, und das Tutorial baut es nach. Ein solcher Adapter
     funktioniert aus Ihrem eigenen Adapterordner, ohne ein neues freemapper.
   - Nein (Kacheln in einem Verzeichnis, Archive, ein STAC-Katalog, ein Download-Index): **Manifest mit
     Backend**. Das Manifest deklariert alles Übrige und nennt ein Rasterbackend, und das ist Rust-Code in
     freemapper. Das Saarland ist das Beispiel: Seine Höhen sind sechs Archive je Modell. Ein Ordner kann
     keinen Code mitbringen, also erreicht eine solche Region die Nutzer erst mit einer neuen Version von
     freemapper.

Die meisten Bundesländer erweisen sich als der zweite Fall von Frage 2. Ihre Dienste sind auf dem Papier
Standard, und dann hat einer eine Eigenheit: Der WFS des Saarlands schreibt seine Attributnamen in
Großbuchstaben und lehnt `SORTBY` ab, und sein WCS hätte einen Parameter zweimal gebraucht. Rechnen Sie mit
einer solchen Sache je Dienst, und nennen Sie sie im Pull-Request.

Drei Grenzen von Format 1, die eine Region zu einem Backend schicken:

- Die drei Rastertabellen sind alle Dienste, oder sie nennen alle dasselbe Backend. Eine einzige Quelle, die
  kein Dienst ist, bedeutet ein Backend für alle drei.
- `params` kann keinen Parameter zweimal enthalten.
- Ein Datum kommt aus einem Dienst, den das Format kennt (ein WFS mit GeoJSON, eine STAC-API, die
  Feature-Info des WMS), oder gar nicht.

## 3. Den Ordner anlegen

Dafür gibt es noch keinen Befehl. Von Hand:

1. Wählen Sie die Kennung: ein Kleinbuchstabe, dann 1 bis 31 Kleinbuchstaben oder Ziffern (`saarland`,
   `bremen`). Die Projektdatei jeder Karte speichert sie, sie bleibt also.
2. Legen Sie einen Ordner mit diesem Namen an. Er enthält `manifest.toml`, die Grenzdatei und nichts, was
   nicht gebraucht wird (höchstens 64 Dateien, keine Verknüpfungen).
3. Beginnen Sie `manifest.toml` mit dem Manifest aus
   [`crates/adapters/de/manifests/`](https://github.com/HenryJobst/freemapper/tree/main/crates/adapters/de/manifests), das Ihrer Region am nächsten ist:
   `saxonyanhalt` für Standarddienste, `saarland` für ein WFS-Kataster mit Backend, `hamburg` für eine OGC API
   mit WFS als Ersatz.
4. Füllen Sie zuerst `[adapter]`, `[net]` und `[[osm.extract]]`. `[net] hosts` zählt jeden Server auf, den der
   Adapter anfragt. Eine Anfrage an einen anderen Host lehnt freemapper ab, auch wenn eine Umleitung oder ein
   Link in einer Antwort dorthin führt.

Arbeiten Sie in einem eigenen Ordner, nicht in Ihrem Adapterordner, bis die Prüfung besteht.

## 4. Die Grenze

freemapper fragt einen Adapter für Kartengebiete innerhalb seiner Grenze, und an einer Landesgrenze
entscheiden die Grenzen, welche Region was liefert. Die Datei ist GeoJSON mit einem Feature, einem `Polygon`
oder `MultiPolygon`, in EPSG:25833 oder in Länge und Breite (EPSG:4326), höchstens 2 MiB groß.

Auch dafür gibt es noch keinen Befehl. So entstand die Grenze des Saarlands:

1. Der Umriss des Bundeslands kam aus dem Kataster selbst: Der WFS hat einen Objekttyp mit den
   Verwaltungseinheiten, und ein Objekt ist das Bundesland. Die Grenzrelation von OpenStreetMap oder VG250 des
   BKG gingen auch; schreiben Sie auf, welche Quelle Sie genommen haben, denn ihre Lizenz gilt für die Datei.
2. Er wurde nach EPSG:25833 umgerechnet.
3. Er wurde auf 25 m vereinfacht, und zwar so, dass das Polygon gültig bleibt (aus 14 037 Stützpunkten wurden
   1 384). QGIS macht das mit „Vereinfachen“ aus den Geometrie-Werkzeugen und „Exportieren > Objekte speichern
   als… > GeoJSON“.
4. Die gemeinsame Grenze wurde mit der Datei des Nachbarn in `crates/adapters/de/manifests/` verglichen. Zwei
   Grenzen, die sich um mehr als einen Streifen überlappen, lassen Karten an der Grenze Objekte doppelt
   bekommen; die Prüfung warnt (`check.shares-ground`), wenn Ihre Region auf einer anderen liegt.

`[boundary] crs` nennt das Koordinatensystem der Datei, `simplify_m`, wie weit Sie vereinfacht haben.
`adapters check` lehnt eine Datei ab, deren Positionen nicht zum genannten Koordinatensystem passen können,
ebenso einen Ring, der sich selbst kreuzt, und warnt bei einer Fläche unter 1 km², was meist die falsche
Einheit bedeutet.

## 5. Das Kataster

Öffnen Sie die Capabilities des Dienstes und suchen Sie die zwei Objekttypen: Gebäude (`GebaeudeBauwerk`) und
Nutzung. Für die Nutzung gibt es zwei Varianten, und die Bundesländer unterscheiden sich darin, welche die
Attribute trägt:

- `Nutzung`: ein Objekt je Fläche einer Nutzung. Sachsen-Anhalt, Saarland.
- `NutzungFlurstueck`: die Nutzung je Flurstücksteil. Hessen, Nordrhein-Westfalen, Niedersachsen, Sachsen.

Holen Sie von jedem ein paar Objekte für ein kleines Rechteck und sehen Sie sie an: Sie brauchen `oid`, die
Klasse in `nutzart` und `bez`, die Funktion eines Gebäudes in `gebnutzbez` und `funktion`. Dann wählen Sie:

- `kind = "wfs"`: ein WFS 2.0, der GML 3.2 antwortet. Setzen Sie `sorted = false`, wenn er `SORTBY=oid` mit
  einer Exception beantwortet. Setzen Sie `square_bez`, wenn das Bundesland nie erfasst, wozu ein Platz dient;
  solche Plätze bleiben dann Kandidaten.
- `kind = "ogc-api-features"`: die zwei Items-Endpunkte. `normalise` sagt, wie der Dienst die Klassennamen
  schreibt: `lgb`, wenn er sie mit Umlauten schreibt wie der von Brandenburg, `hamburg-ascii`, wenn er
  `Wohnbauflaeche` schreibt. Passt keines, braucht die Region eine neue Voreinstellung, und das ist Code. Ein
  WFS unter `[alkis.fallback]` wird gefragt, wenn die API ausfällt.

Hat das Bundesland außerdem einen WFS mit dem vollen ALKIS (NAS), ergänzt `[[extra.nas]]` Bauwerke,
Vegetationsmerkmale und kleine Gewässer. Das Baumkataster einer Stadt auf einem Opendatasoft-Portal gehört in
`[[extra.trees]]`.

## 6. Die Raster

### Als Dienste

Für jede Tabelle brauchen Sie die Adresse, den Namen der Ebene oder des Coverage und die
Grenzen einer Anfrage:

- `[orthophoto]`, `kind = "wms"`: die Ebene mit dem Echtfarbenbild in 20 cm und `max_px`, die längste Seite,
  die der Server in einem GetMap liefert (seine Capabilities nennen dafür manchmal `MaxWidth` und
  `MaxHeight`; Sachsen-Anhalt wird mit 4000 gefragt).
- `[terrain]`, `[surface]`, `kind = "wcs"`: die Coverage-Kennung, ihre Ausdehnung im Koordinatensystem des
  Adapters (`envelope`, aus DescribeCoverage) und `max_m`, die längste Seite einer Anfrage in Metern. Fragen
  Sie einmal von Hand ein kleines Rechteck ab und sehen Sie die Antwort an: ein TIFF oder ein
  `multipart/related`-Körper mit dem TIFF darin (`multipart = true`); 1 m je Pixel oder ein gröberes Raster,
  das einen Parameter in `params` braucht.
- `date`: wo sich das Flugdatum oder das Datum des Modells erfragen lässt. Ohne das tragen die Kartennotizen
  kein Datum.

### Als Backend

Ist eine Quelle kein Dienst, sagen die drei Tabellen `backend = "<Name>"` und sonst nichts,
und der Code liegt im Adapter-Crate des Landes, für Deutschland `crates/adapters/de`. Ein Backend
implementiert ein Trait:

```rust
impl RasterBackend for MyBackend {
    fn provider(&self) -> &ProviderInfo {
        &self.info
    }

    fn fetch(
        &self,
        env: &mut AdapterEnv<'_>,
        project: &mut dyn RasterProject,
        bbox: &BoundingBox,
    ) -> Result<(), AdapterError> {
        // The orthophoto, the terrain model and the surface model of `bbox` (the map area
        // plus buffer, in the CRS of the map): into the project folder, with their records,
        // saving the project after each part. `env.fetcher` is the only way to the network,
        // `env.cache` holds what other maps can reuse, `env.mode` says whether the run is
        // offline, and with `env.skip_surface` the surface model is left out.
        Ok(())
    }
}
```

Lesen Sie [`crates/adapters/de/src/lvgl.rs`](https://github.com/HenryJobst/freemapper/blob/main/crates/adapters/de/src/lvgl.rs) (Saarland), bevor Sie
eines schreiben. Die Datei zeigt, was jedes Backend hat: das Luftbild aus einem WMS über die gemeinsame
Hilfsfunktion, Kacheln, die mit Bereichsanfragen aus Archiven gelesen und im Zwischenspeicher aufgehoben
werden, ein fehlendes Oberflächenmodell als Warnung statt als Fehler, und Tests, die ohne Netz gegen
künstliche Archive laufen. Registriert wird das Backend unter seinem Namen in
`crates/adapters/de/src/states/mod.rs`.

Ein eingebautes Bundesland hinzuzufügen ist noch nicht eine Zeile. Das Saarland hat etwa zwanzig Stellen im
Crate berührt, weil die Bundesländer ein Enum sind, dazu einige Tests, die die Bundesländer zählen;
[`saarland-reference.md`](https://github.com/HenryJobst/freemapper/blob/main/docs/regions/saarland-reference.md) hat die Liste, und Commit `dde0b5e` zeigt die
Stellen. Wenn Sie kein Rust schreiben, schicken Sie den Manifestordner und die Recherche und sagen Sie es dazu.

## 7. Quellenangabe und Lizenz

Jede Karte trägt die Quellenangaben ihrer Daten. Ein Fehler hier trifft andere Leute, nämlich alle, die die
Karte weitergeben. Arbeiten Sie deshalb aus den Quellen:

1. Suchen Sie die Lizenz jedes Produkts im Katalogeintrag des Amts, und lesen Sie `Fees` und
   `AccessConstraints` in den Capabilities jedes Dienstes. Sagt ein Dienst mit eigenen Worten, dass das
   Herunterladen einen Vertrag braucht, dann nutzen Sie ihn nicht, gleich was der Katalog sagt. Die zwei
   INSPIRE-WCS-Dienste des Saarlands blieben aus diesem Grund außen vor. Verweisen die Capabilities nur auf
   allgemeine Bedingungen, entscheiden die Nutzungsbedingungen des Amts; zitieren Sie den Satz im
   Pull-Request.
2. Übernehmen Sie den Namen des Bereitstellers genau so, wie das Amt ihn vorschreibt
   (`© GeoBasis-DE / LVermGeo ST`, `© GeoBasis DE/LVGL-SL (Jahr)`), mit Leerzeichen und Klammern.
3. Schreiben Sie `[attribution] text` mit dem, was die Lizenz verlangt:

   | Lizenz | Die Zeile braucht |
   |---|---|
   | `dl-de/by-2-0` | den Namen des Bereitstellers, `dl-de/by-2-0`, den Hinweis, dass die Daten verändert wurden (`Daten geändert`), und in `note` die Adresse des Lizenztexts |
   | `cc-by-4.0` | den Namen des Bereitstellers, `CC BY 4.0`, einen Änderungshinweis und in `note` die Adresse des Lizenztexts |
   | `dl-de/zero-2-0`, `cc0-1.0` | nichts; nennen Sie den Bereitsteller trotzdem |
   | `odbl` | den Namen des Bereitstellers und `ODbL` |
   | `other` | was ihr Text sagt; der Text gehört in `licence_text` |

   `{year}` steht für das Jahr des Downloads. Die Zeilen für OpenStreetMap und für freemapper selbst ergänzt
   das Programm.
4. `[preview] attribution` ist die Quellenangabe unter dem Luftbild der Gebietsseite; sie hat keinen
   Änderungshinweis, weil dieses Bild so gezeigt wird, wie es kommt.

Lassen Sie weg, was nicht offen ist, auch wenn es die bessere Karte gäbe. Ein Produkt hinter einem Vertrag
wird nicht in freemapper eingebaut.

## 8. Testen

`freemapper adapters check <Ordner>` ist der Test. Ohne Schalter fragt der Befehl keinen Server und sagt, ob
der Ordner laden würde, mit allen Befunden auf einmal: Datei, Zeile, Schlüssel und ein Code, den die
[Referenz](manifest-v1.de.md#befunde) erklärt. Die Texte der Befunde sind in jeder Programmsprache englisch.

```sh
freemapper adapters check ~/work/saarland
freemapper adapters check ~/work/saarland --live --bbox 6.9930,49.2330,6.9952,49.2344
```

Mit `--live` holt er, was eine Karte des Rechtecks bräuchte, von den Hosts, die das Manifest nennt, und gibt
je Fähigkeit eine Zeile aus: die Raster, Gebäude, Nutzung, das volle ALKIS, den OSM-Auszug. Lassen Sie ihn für
drei Rechtecke von etwa 150 m laufen:

- eine Ortsmitte, wo jede Quelle Daten hat;
- einen Ort am Rand der Region, wo sich Dienste oft anders verhalten;
- einen Ort, an dem zwei Ihrer Quellen aufeinandertreffen, wenn es einen gibt (zwei Landkreise, zwei
  Bildflugblöcke).

Machen Sie danach eine echte Karte mit dem Adapter in Ihrem Adapterordner und sehen Sie sie an: Gebäude dort,
wo das Luftbild sie zeigt, Höhenlinien, die dem Gelände folgen, die Quellenangabe in `ATTRIBUTION.txt`.

Für ein eingebautes Bundesland bekommt der Code außerdem Tests, die ohne Netz laufen: aufgezeichnete Antworten
der Dienste, auf wenige Objekte gekürzt, in `tests/fixtures/`, und für ein Backend künstliche Archive oder
Kacheln. [`fixtures.md`](fixtures.md) (englisch) beschreibt das Format einer Aufzeichnung. Aufgezeichnete
Antworten werden mit dem Repository veröffentlicht: keine Adresse mit `@`, kein Name einer Person, keine
E-Mail-Adresse in einer Anfrage oder einer Datei.

## 9. Einreichen

Öffnen Sie einen Pull-Request auf <https://github.com/HenryJobst/freemapper>. Für ein deutsches Bundesland
gehört der Ordner nach `crates/adapters/de/manifests/<Kennung>/`. Die Prüfliste:

- [ ] `freemapper adapters check <Ordner>` besteht; die Ausgabe steht im Pull-Request.
- [ ] `--live` besteht für drei Rechtecke; die Befehle und die Ausgabe stehen im Pull-Request.
- [ ] Die Lizenz jedes Produkts ist genannt, mit der Adresse der Seite oder des Katalogeintrags, der sie
      nennt, und der Text der Quellenangabe entspricht dem, was diese Seite vorschreibt.
- [ ] `maintainers` nennt, wer antwortet, wenn sich ein Dienst ändert (ein GitHub-Name, keine
      E-Mail-Adresse), und `last_verified` ist der Tag Ihrer Live-Prüfung.
- [ ] `[net] hosts` nennt die Hosts, die der Adapter braucht, und keine anderen.
- [ ] Die Quelle der Grenze ist genannt.
- [ ] Keine personenbezogenen Daten in aufgezeichneten Antworten, Dateinamen oder im Manifest.
- [ ] Für ein neues Bundesland: ein Beispielprojekt in `examples/`, eine Zeile in der Regionentabelle des
      Handbuchs (deutsch und englisch), eine Zeile in beiden READMEs und ein Changelog-Eintrag in beiden
      Sprachen. Der Maintainer hilft dabei.

Worauf der Maintainer sieht: zuerst auf die Lizenz, dann auf die Hosts, dann darauf, ob die Live-Prüfung an
einem anderen Tag und auf einem anderen Rechner besteht, dann auf die Karte des Beispiels. Ein Manifest, das
nur sein Autor prüfen kann, wird nicht übernommen.

## 10. Am Leben halten

Dienste ziehen um, und Flugjahre wechseln. Was ein Adapter über die Zeit braucht:

- Lassen Sie `adapters check --live` laufen, wenn eine Karte in Ihrer Region scheitert, und mindestens einmal
  im Jahr. Setzen Sie `last_verified` auf diesen Tag. Die Prüfung warnt, wenn das Datum älter als zwölf Monate
  ist.
- Zieht ein Dienst an eine andere Adresse, ändern Sie die URL und, wenn der Host neu ist, `[net] hosts`.
  Erhöhen Sie `revision`, wenn die Änderung die Daten einer Karte verändert: ein anderer Dienst, ein anderes
  Produkt, ein anderes Jahr.
- Bei einem eingebauten Bundesland können sich Nutzer bis zur nächsten Version selbst helfen: Die Meldung
  über die abgelehnte Adresse nennt den Befehl `freemapper adapters allow-host <Kennung> <Host>`. Bitten Sie
  sie, den Umzug zu melden.
- Ein neuer Schlüssel des Formats braucht ein neueres freemapper. Setzen Sie `min_freemapper` auf die
  Version, die den Schlüssel eingeführt hat, damit ein älteres Programm „braucht ein neueres freemapper“ sagt
  und nicht „unbekannter Schlüssel“.

Eine regelmäßige Live-Prüfung aller Adapter gibt es noch nicht; kaputte Dienste bemerken die Leute, die Karten
machen. Ein Adapter im Repository, dessen Maintainer nicht antwortet, bleibt, solange er funktioniert. Geht er
kaputt und repariert ihn niemand, repariert ihn der Maintainer von freemapper oder nimmt ihn mit einem
Changelog-Eintrag heraus.
