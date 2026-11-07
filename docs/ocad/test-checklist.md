# OCAD-Test / OCAD test

Prüfliste für einen von freemapper erzeugten Ordner `ocad/` (ganzen Ordner kopieren). Nicht in OCAD
geprüft ist bisher alles hier; bitte Ergebnisse als Issue oder Nachricht zurückmelden.

Checklist for an `ocad/` folder written by freemapper (copy the whole folder). Nothing here has been
checked in OCAD yet; please report results as an issue or message.

| # | Prüfen / Check | Erwartet / Expected |
|---|---|---|
| 1 | `<name>.ocd` in OCAD öffnen (aktuelle Version, optional OCAD 12) / open in OCAD | öffnet ohne Warnung / opens without warning |
| 2 | *Optimize/Repair* | 0 beschädigte Objekte / 0 damaged objects |
| 3 | Koordinatensystem / coordinate system | ETRS89 / UTM Zone 33N, Kartenmitte plausibel / map centre plausible |
| 4 | Luftbild (`orthophoto.jpg`) | deckt sich mit Gebäuden, Wegen und Nordlinien / matches buildings, paths, north lines |
| 5 | Schummerung, Hangneigung, nDOM | vorhanden, ausgeblendet, gedimmt / present, hidden, dimmed |
| 6 | `<name>-candidates.ocd` als Hintergrundkarte / as background map | deckungsgleich eingeblendet; sonst von Hand öffnen / aligned; else open by hand |
| 7 | *Symbol › Farben und Symbole laden aus…* mit ISSprOM 2019 (Sprint) bzw. IOF-Schulsatz 2019 | Platzhalter werden nach Nummer ersetzt / placeholders replaced by number |
| 8 | Nach dem Laden / after loading: Randangaben (Symbole 991–994) | erhalten oder verloren? / kept or dropped? |
| 9 | Schulsatz: Hecke 410.002, 321, 541–545 | richtige Symbole / right symbols |
| 10 | Texte (Titel, Maßstab, Namensnennung mit ©, ß, Umlauten) | richtig platziert und dargestellt / placed and shown correctly |
| 11 | Karten-Notizen / map notes | Namensnennung und freemapper-Link / attribution and freemapper link |
| 12 | Flächen mit Löchern (Gebäude mit Innenhof; die Strausberger Beispiele haben keine) / areas with holes (the Strausberg samples have none) | Löcher bleiben frei / holes stay empty |
| 13 | Farbreihenfolge / colour order | wie im Symbolsatz / as in the symbol set |
