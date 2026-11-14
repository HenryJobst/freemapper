# Example projects

[Deutsch](README.md)

Each supported state has an example: an area from which freemapper can derive a lot (an old town, a palace or a
park with relief, dense buildings, stairs, walls and hedges). All examples are sprint maps at 1:4000 (ISSprOM 2019),
made with `freemapper run --pdf --pdf-candidates --ocad` and the default content profile `balanced`; Sanssouci,
Viktoriapark, Monschau, Cochem and Saarlouis also with `--content-set vegetation=candidates`. The PDF of each
example is the training map with every candidate, because it shows what freemapper can derive. The map alone shows
less; the [comparison page](compare.en.md) shows the same area in the three content profiles.

| State | Area | Map objects | Candidates | Symbols | Preview |
|---|---|---:|---:|---:|---|
| Brandenburg | [Park Sanssouci, Potsdam](brandenburg-sanssouci/): vineyard terraces, parterres and hedges, palace | 2676 | 623 | 39 | <a href="brandenburg-sanssouci/brandenburg-sanssouci-candidates.pdf"><img src="brandenburg-sanssouci/preview.webp" width="220" alt=""></a> |
| Berlin | [Viktoriapark, Kreuzberg](berlin-viktoriapark/): the Kreuzberg hill with its waterfall, 19th-century blocks | 3443 | 624 | 45 | <a href="berlin-viktoriapark/berlin-viktoriapark-candidates.pdf"><img src="berlin-viktoriapark/preview.webp" width="220" alt=""></a> |
| Saxony | [Bautzen old town](sachsen-bautzen/): the old town on the rock above the Spree, steep valley sides | 2527 | 988 | 39 | <a href="sachsen-bautzen/sachsen-bautzen-candidates.pdf"><img src="sachsen-bautzen/preview.webp" width="220" alt=""></a> |
| Saxony-Anhalt | [Schlossberg Quedlinburg](sachsen-anhalt-quedlinburg/): World Heritage old town, castle hill, narrow lanes | 2947 | 585 | 38 | <a href="sachsen-anhalt-quedlinburg/sachsen-anhalt-quedlinburg-candidates.pdf"><img src="sachsen-anhalt-quedlinburg/preview.webp" width="220" alt=""></a> |
| Hamburg | [Treppenviertel Blankenese](hamburg-blankenese/): the steep Elbe bank, stairs and lanes | 1780 | 992 | 31 | <a href="hamburg-blankenese/hamburg-blankenese-candidates.pdf"><img src="hamburg-blankenese/preview.webp" width="220" alt=""></a> |
| Hesse | [Marburg old town and castle](hessen-marburg/): the steep upper town with stepped lanes, castle hill | 3051 | 1015 | 35 | <a href="hessen-marburg/hessen-marburg-candidates.pdf"><img src="hessen-marburg/preview.webp" width="220" alt=""></a> |
| Schleswig-Holstein | [Lübeck old town and Holstentor](schleswig-holstein-luebeck/): the medieval old-town island with its passages and courtyards | 3345 | 1365 | 35 | <a href="schleswig-holstein-luebeck/schleswig-holstein-luebeck-candidates.pdf"><img src="schleswig-holstein-luebeck/preview.webp" width="220" alt=""></a> |
| Mecklenburg-Vorpommern | [Wismar old town](mecklenburg-vorpommern-wismar/): World Heritage old town with St. George's church and the market square | 3396 | 1416 | 35 | <a href="mecklenburg-vorpommern-wismar/mecklenburg-vorpommern-wismar-candidates.pdf"><img src="mecklenburg-vorpommern-wismar/preview.webp" width="220" alt=""></a> |
| North Rhine-Westphalia | [Monschau old town](nordrhein-westfalen-monschau/): half-timbered old town in the deep Rur valley | 2456 | 1299 | 40 | <a href="nordrhein-westfalen-monschau/nordrhein-westfalen-monschau-candidates.pdf"><img src="nordrhein-westfalen-monschau/preview.webp" width="220" alt=""></a> |
| Lower Saxony | [Großer Garten Herrenhausen, Hanover](niedersachsen-herrenhausen/): baroque garden with hedges, parterres and fountain basins, next to the Georgengarten | 2400 | 333 | 38 | <a href="niedersachsen-herrenhausen/niedersachsen-herrenhausen-candidates.pdf"><img src="niedersachsen-herrenhausen/preview.webp" width="220" alt=""></a> |
| Rhineland-Palatinate | [Reichsburg and Cochem old town](rheinland-pfalz-cochem/): the castle hill with the Reichsburg, the old town on the steep Moselle bank, Cond across the river | 2862 | 1024 | 44 | <a href="rheinland-pfalz-cochem/rheinland-pfalz-cochem-candidates.pdf"><img src="rheinland-pfalz-cochem/preview.webp" width="220" alt=""></a> |
| Saarland | [Saarlouis old town and fortress](saarland-saarlouis/): Vauban's hexagonal fortress town, the old arm of the Saar with the Vauban island, ramparts in the Stadtgarten | 1688 | 799 | 40 | <a href="saarland-saarlouis/saarland-saarlouis-candidates.pdf"><img src="saarland-saarlouis/preview.webp" width="220" alt=""></a> |
| Baden-Württemberg | [Heidelberg old town and castle](baden-wuerttemberg-heidelberg/): the old town on the Neckar with the Old Bridge, the castle and its garden on the steep slope of the Königstuhl | 2858 | 899 | 43 | <a href="baden-wuerttemberg-heidelberg/baden-wuerttemberg-heidelberg-candidates.pdf"><img src="baden-wuerttemberg-heidelberg/preview.webp" width="220" alt=""></a> |
| Bremen | [Old town of Bremen and Wallanlagen](bremen-altstadt/): the old town between the Weser and the town moat with the market square and the Schlachte, the Wallanlagen with the mill on the rampart, and the Teerhof across the river | 2498 | 534 | 39 | <a href="bremen-altstadt/bremen-altstadt-candidates.pdf"><img src="bremen-altstadt/preview.webp" width="220" alt=""></a> |
| Bavaria | [Old town of Regensburg with the Stone Bridge](bayern-regensburg/): the World Heritage old town on the Danube with the cathedral and its lanes, the Stone Bridge, the river islands and Stadtamhof on the other bank | 2959 | 453 | 38 | <a href="bayern-regensburg/bayern-regensburg-candidates.pdf"><img src="bayern-regensburg/preview.webp" width="220" alt=""></a> |

**Map objects** are the objects of the OOM map. With `balanced` these are the objects of the cadastre, the registers
and the street survey (buildings, streets with curbs, water, parks, forest, canopies and passages), plus walls,
fences, hedges, trees, stairs and paths from OSM as well, and the contours. **Candidates** are what stays uncertain,
mostly out-of-bounds areas (520) and control sites (531), plus banks, cliffs and knolls from the terrain model (group
`landforms`). They come as a separate template (`templates/candidates.omap`). The PDF `<name>-candidates.pdf` draws
them too; `<name>.pdf` from `--pdf` shows only the map objects. Both come from OSM, from ALKIS and from the terrain
model, in Berlin also from the Geoportal Berlin, and in Potsdam from the city map tree points (Bäume der Stadtkarte).
**Symbols** counts the distinct symbols in
both together. Contours and north lines stop at ways, so they count in pieces.

The group `vegetation` is off in every profile, because the point clouds are large. Five examples switch it on as
candidates: Sanssouci, Viktoriapark, Monschau, Cochem and Saarlouis. There the training map shows what the laser points say
about the vegetation: runnable forest (405) under the tree crowns, open land (401) and the green shades 406, 408
and 410. Sanssouci has 153 forest areas, 78 areas of open land and 74 green areas, Monschau 290, 218 and 264,
Cochem 218, 115 and 186, Saarlouis 286, 105 and 90, Viktoriapark 56, 51 and 47. The park turns white where trees
stand and stays yellow where there is lawn. Berlin's flight of 2021 was flown without leaves, so the green shades
are weaker there. Areas that shall not be entered (520) hold no vegetation: the areas end at them, as they do at
buildings, water and paved areas. The group does not change the map objects. Saxony offers no point clouds at
present, and freemapper does not read any for the other states yet.

Rivers, canals, ponds and harbour basins from ALKIS are on the map as uncrossable water (301.1) with a bank line
(301.6). Ponds, fountains, springs and streams from OSM go onto the map with `balanced` too; bridges from OSM stay
candidates. The water has no contours and no fountain points, which would mostly be the jets in a basin. Where ALKIS
records a bridge as road area, the water ends at the bridge with a bank line across the river; delete that line in
OOM.

Canopies and carports from ALKIS are on the map as 522 in every state, and with `balanced` so are passages and arcades
through buildings. Only passages nobody can reach stay candidates. Lübeck has 93 canopies and 185 passages and
arcades from ALKIS, Wismar 113 canopies and carports. In Saxony and Saxony-Anhalt ALKIS has neither canopies nor
passages. The canopies there come from OSM only, 26 in Bautzen and 15 in Quedlinburg.

In Brandenburg freemapper also loads the full ALKIS (NAS) from the LGB. In Sanssouci that gives 270 wall lines (515),
107 fences (518), 133 hedges (410.1) and 125 embankment tops (104), with `balanced` all on the map. ALKIS often
records both edges of a wall. Where they face each other and lie closer than 0.55 mm on paper, freemapper draws one
line in the middle; Sanssouci has 14 of these. Thicker walls and edges that face the same way stay double; delete the
second line in OOM. Where OSM has the same wall, hedge or fence, only the ALKIS line stays, and it names the OSM line
in `freemapper:also`. Where a line with another symbol runs parallel and too close, such as an OSM retaining wall on
a terrace wall or a fence beside a wall, the weaker line loses that stretch. Wall stretches along a building outline
and trees within 1 m of a hedge, wall or fence are dropped. In Sanssouci this leaves only a few parallel line pairs
closer than 1 m. The Potsdam city map tree points are a tree cadastre as in Berlin. Of them, 773 stay points: 658 are
on the map, and 115 lie in out-of-bounds areas (520) and stay candidates. The others lie in tree group areas or are
dropped by the thinning. OSM trees closer than 2 m to a cadastre tree are dropped.

In Berlin, freemapper loads more layers from the Geoportal Berlin. Stairs, walls, fences, embankment edges, monuments
and passages go onto the map with `balanced`, and so do the trees from the tree cadastre, 1926 of them in the
Viktoriapark. The Berlin 1:5000 map (K5) comes as the template `templates/k5.png`.

Berlin projects also get the street survey (Straßenbefahrung, recorded 2014 to 2016), the registers of street
lamps, ornamental fountains and drinking fountains, and the building heights from the Umweltatlas. With `balanced`
their objects are on the map. In the Viktoriapark these are 76 curbs (501.1), 73 surveyed paved areas (501.5) and
five small green strips along the curb that freemapper cuts out of one ALKIS street area. The areas are nearly all
sidewalks and the paved paths in the park. Traffic barriers (518), walls (515), bus shelters (522), and street pumps
and ornamental fountains (311) come as well. Advertising columns and street lamps stay candidates as control sites
(531). Where a survey barrier
lies on an OSM fence, only the barrier stays, and it names the fence in `freemapper:also`. Of the 602 buildings, 599
carry the footprint date from the 2022 building heights as `freemapper:currency`.

Allotment gardens (Kleingärten) are out-of-bounds candidates (520) in every state. In Sanssouci this covers one
garden colony on the northern edge, in Wismar and Herrenhausen one more area each.

Point symbols must not touch. Two points keep at least 0.15 mm apart; otherwise the larger one stays and names the
other in the tag `freemapper:generalized`. Where three or more trees stand closer than 2.5 mm (10 m) or their crowns
almost touch, freemapper turns them into an area: forest (405) at 50 % crown cover or more, otherwise open land with
scattered trees (402), and 501.3 on paving from 9 trees. These areas leave paths and paving clear. Rows of trees
along streets and avenues stay points. Sanssouci gets 38 areas 402 this way, because the Potsdam trees have no crown
and 5 m crowns do not cover half of a group. The Viktoriapark, with measured crowns, gets 42 forests 405 and 3 areas
402. Of its 1926 cadastre trees, 1179 stay points; the others lie in tree group areas or are dropped by the thinning.

Parks are yellow open land (401). ALKIS parks, green spaces and grassland are map objects, in Sanssouci 27 areas,
in the Viktoriapark 5. ALKIS woods become forest (405), in Monschau 153 areas. Under everything lies the walkable
area, one brown area over the whole sheet (symbol 990, map part `base`). It fills whatever nothing else shows. The PDF with candidates leaves out points inside a 520 candidate; in the
template they carry `freemapper:hidden=520`.

In the Viktoriapark the stairs on the plinth of the Nationaldenkmal lead up to the monument. Wide stairs such as the
stands at the stadium come as parallel 532.9 lines tagged `freemapper:size=wide`. Fences and walls that both ALKIS and
OSM have appear once on the map, as the ALKIS line, which names the OSM line in the tag `freemapper:also`.

## Forest maps

One example of the map kind `forest` (ISOM 2017-2) at 1:10 000 for every state, for a quick training in a forest:

| State | Area | Map objects | Candidates | Symbols | Preview |
|---|---|---:|---:|---:|---|
| Brandenburg | [Rauener Berge and Markgrafensteine](brandenburg-rauener-berge/): moraine hills above the Scharmützelsee with the Markgrafensteine, large erratic boulders | 385 | 37 | 20 | <a href="brandenburg-rauener-berge/brandenburg-rauener-berge-candidates.pdf"><img src="brandenburg-rauener-berge/preview.webp" width="220" alt=""></a> |
| Berlin | [Dammheide, Treptow-Köpenick](berlin-dammheide/): a small flat forest with a dense path network, settlements around it | 1179 | 166 | 27 | <a href="berlin-dammheide/berlin-dammheide-candidates.pdf"><img src="berlin-dammheide/preview.webp" width="220" alt=""></a> |
| Saxony | [Bastei, Saxon Switzerland](sachsen-bastei/): sandstone rocks above the Elbe, more than 2000 rock objects from the terrain model | 3847 | 571 | 34 | <a href="sachsen-bastei/sachsen-bastei-candidates.pdf"><img src="sachsen-bastei/preview.webp" width="220" alt=""></a> |
| Saxony-Anhalt | [Bode valley near Thale](sachsen-anhalt-bodetal/): the deep valley of the Bode in the Harz | 893 | 76 | 31 | <a href="sachsen-anhalt-bodetal/sachsen-anhalt-bodetal-candidates.pdf"><img src="sachsen-anhalt-bodetal/preview.webp" width="220" alt=""></a> |
| Hamburg | [Haake, Harburger Berge](hamburg-haake/): a hilly forest with a dense path network | 549 | 33 | 23 | <a href="hamburg-haake/hamburg-haake-candidates.pdf"><img src="hamburg-haake/preview.webp" width="220" alt=""></a> |
| Hesse | [Großer Feldberg, Taunus](hessen-feldberg/): the summit and slopes of the highest mountain of the Taunus | 657 | 39 | 27 | <a href="hessen-feldberg/hessen-feldberg-candidates.pdf"><img src="hessen-feldberg/preview.webp" width="220" alt=""></a> |
| Schleswig-Holstein | [Hüttener Berge](schleswig-holstein-huettener-berge/): moraine hills with forest, open land and small lakes | 352 | 72 | 24 | <a href="schleswig-holstein-huettener-berge/schleswig-holstein-huettener-berge-candidates.pdf"><img src="schleswig-holstein-huettener-berge/preview.webp" width="220" alt=""></a> |
| Mecklenburg-Vorpommern | [Granitz, Rügen](mecklenburg-vorpommern-granitz/): beech forest around the Granitz hunting lodge | 321 | 96 | 24 | <a href="mecklenburg-vorpommern-granitz/mecklenburg-vorpommern-granitz-candidates.pdf"><img src="mecklenburg-vorpommern-granitz/preview.webp" width="220" alt=""></a> |
| North Rhine-Westphalia | [Drachenfels, Siebengebirge](nordrhein-westfalen-siebengebirge/): rocks, vineyards and forest above the Rhine | 1158 | 269 | 37 | <a href="nordrhein-westfalen-siebengebirge/nordrhein-westfalen-siebengebirge-candidates.pdf"><img src="nordrhein-westfalen-siebengebirge/preview.webp" width="220" alt=""></a> |
| Lower Saxony | [Harz near Bad Harzburg](niedersachsen-bad-harzburg/): steep slopes on both sides of a stream valley | 677 | 51 | 32 | <a href="niedersachsen-bad-harzburg/niedersachsen-bad-harzburg-candidates.pdf"><img src="niedersachsen-bad-harzburg/preview.webp" width="220" alt=""></a> |
| Rhineland-Palatinate | [Dahner Felsenland](rheinland-pfalz-dahn/): red sandstone rocks in the Palatinate Forest | 641 | 90 | 33 | <a href="rheinland-pfalz-dahn/rheinland-pfalz-dahn-candidates.pdf"><img src="rheinland-pfalz-dahn/preview.webp" width="220" alt=""></a> |
| Saarland | [Cloef at the Saarschleife](saarland-cloef/): a steep slope with rocks above the Saar loop | 492 | 56 | 27 | <a href="saarland-cloef/saarland-cloef-candidates.pdf"><img src="saarland-cloef/preview.webp" width="220" alt=""></a> |
| Baden-Württemberg | [Königstuhl, Heidelberg](baden-wuerttemberg-koenigstuhl/): Heidelberg's own mountain, with an observatory and many paths | 843 | 47 | 28 | <a href="baden-wuerttemberg-koenigstuhl/baden-wuerttemberg-koenigstuhl-candidates.pdf"><img src="baden-wuerttemberg-koenigstuhl/preview.webp" width="220" alt=""></a> |
| Bremen | [Bürgerpark and Stadtwald](bremen-buergerpark/): a flat park and forest with ditches, lakes and a dense path network | 1913 | 420 | 32 | <a href="bremen-buergerpark/bremen-buergerpark-candidates.pdf"><img src="bremen-buergerpark/preview.webp" width="220" alt=""></a> |
| Bavaria | [Schmausenbuck, Nürnberger Reichswald](bayern-schmausenbuck/): a sandstone hill in the Reichswald above the zoo of Nuremberg, slopes with a dense path network, two power lines | 670 | 81 | 35 | <a href="bayern-schmausenbuck/bayern-schmausenbuck-candidates.pdf"><img src="bayern-schmausenbuck/preview.webp" width="220" alt=""></a> |

The areas are about 1.2 × 0.9 km and were chosen by relief and path network. Made with
`freemapper init <folder> --geojson <rectangle> --kind forest --scale 10000 --title "<title>"` and
`run --pdf-candidates --ocad`. Berlin and Bremen take their outline from OSM (`--osm relation/4513841` for the
Dammheide, `--osm way/664446849` for the Bürgerpark). In Brandenburg, North Rhine-Westphalia and Saarland
`--content-set vegetation=candidates` was added: the vegetation from laser point clouds is in the candidates map
there. Rhineland-Palatinate lacks it, because the state office's Metalink file names other sizes and checksums for
the tiles near Dahn than the files on the server have (November 2026); for Bavaria freemapper does not read the
point clouds yet.

The paths come from OSM and go onto the map in the path classes 502 to 507; the forest is white paper. Settlements
and gardens are olive green (520, not to be entered). Rocks, earth banks and knolls from the terrain model are
candidates. In the flat areas in Berlin and Bremen the contours lie every 2.5 m, elsewhere every 5 m;
`freemapper.toml` names the 2.5 m as `[contours] interval_m`.

## What an example contains

| File | Content |
|---|---|
| `<name>.omap` | the OOM starter project (OpenOrienteering Mapper) |
| `templates/candidates.omap` | the candidates as a template |
| `ocad/<name>.ocd`, `ocad/<name>-candidates.ocd` | the OCAD starting point (OCAD 12) |
| `<name>-candidates.pdf` | the training map: map objects and every candidate, without a scale bar |
| `preview.webp` | preview of this PDF |
| `<name>.pdf`, `<name>-safe.pdf`, `<name>-full.pdf`, `preview-safe.webp`, `preview-balanced.webp`, `preview-full.webp` | only in Cochem: the same area in the three content profiles, without candidates, and the previews, see the [comparison page](compare.en.md) |
| `freemapper.toml` | the project: area, scale and the fetched data versions |
| `ATTRIBUTION.txt` | the attribution of the data sources |

The raster templates (orthophoto, hillshade, slope, nDSM, in Berlin the K5) and the raw data (`data/`) are not in the
repository, because they take over 20 MB per example. To get them back:

```sh
freemapper run --ocad --pdf --pdf-candidates examples/sachsen-anhalt-quedlinburg
```

freemapper fetches the data again, computes the templates and keeps the existing maps (`.omap`, `.ocd`) unchanged.
The map then opens in OOM and OCAD with all templates. `--pdf` also writes the map without candidates.

## How the areas were chosen

For each state, five to six candidates were made as sprint maps, among them parks, schools, old towns and palace
grounds. The distinct symbols and the objects were counted; the best two or three areas of each state were then
compared on the training map. The areas that show the most won: varied buildings, relief and many OSM candidates.

In Lower Saxony the Großer Garten won, although before the content profiles its map had the fewest map objects. Its
hedges, parterres, rows of trees and fountains gave the most candidates per map object. With `balanced` they are on
the map, among them 368 hedges (410.1), 1124 trees (418) and 17 fountains (311), and 333 candidates are left.

In Rhineland-Palatinate eight rectangles ran: Koblenz at the Deutsches Eck, Trier, Speyer, Mainz, Bad Kreuznach and
Cochem. Bad Kreuznach and Cochem ran twice, because the first rectangle in Bad Kreuznach missed the Nahe and the one
in Cochem missed the Reichsburg. Speyer and Cochem both reached 24 symbols. Cochem won because only this area has
strong relief: dense contours on the castle hill and the valley sides, plus the Moselle and the narrow old town.
Speyer is flat. Koblenz shows the Rhine and the Moselle at the Deutsches Eck, but it reached only 23 symbols and has
little relief.

In Saarland two rectangles ran, both in Saarlouis. The second lies 180 m further west and takes in the Vauban
island in the old arm of the Saar and the ramparts in the Stadtgarten. Saarlouis was the first example made with
the content profile `balanced`, which draws the objects of the cadastre and the registers straight onto the map. Its
map has 31 symbols, among them 21 stairs from OpenStreetMap, and 799 candidates are left, most of them vegetation.
Since the land uses of Saarland are read in both spellings, 109 street areas and the Saar from ALKIS are on the
map. All examples have been rebuilt with this profile since.

In Baden-Württemberg three rectangles ran: Heidelberg, the old town of Freiburg with the Schlossberg, and the old
town of Tübingen with its castle and the island in the Neckar. Tübingen had the most map objects (3910), Heidelberg
the most symbols on the map (40) and the most candidates (903). Heidelberg won because its area shows the widest
range: the castle with its terraces and garden on the steep slope, below it the dense old town and the Neckar.

In Bremen four rectangles ran: the old town with the Wallanlagen, the university, the Bürgerpark and the
Havenwelten in Bremerhaven. The university had the most symbols (44 with the candidates), the old town the most
map objects (2498) and candidates (534). The old town won: the town moat and the Weser, the rampart as a park
with contours and the dense old town are on one sheet, and 64 stairs, 13 retaining walls and 16 embankment tops
come from the full ALKIS, which only Brandenburg has besides.

In Bavaria one rectangle ran for the sprint map, the old town of Regensburg. The arms of the Danube with their
islands, the Stone Bridge and the dense old town are on one sheet. Bavaria has no service for its cadastre; the
1821 buildings and 144 canopies come from the LoD2 tiles, the land use from the file of the city of Regensburg.
The area lies east of 12° E and is in UTM zone 32 like all of Bavaria. For the forest map two rectangles ran at
the Schmausenbuck; half of the southern one was the zoo.

## Licences

The maps contain derived official geodata and OpenStreetMap data. The attribution is in each `ATTRIBUTION.txt`, in
the PDFs' peripheral information and in the map notes:

- OpenStreetMap: © OpenStreetMap contributors, ODbL;
- Brandenburg: © GeoBasis-DE/LGB, dl-de/by-2-0; Potsdam: Landeshauptstadt Potsdam / Bäume der Stadtkarte, dl-de/zero-2-0; Berlin: © Geoportal Berlin, dl-de/zero-2-0;
- Saxony: GeoSN, dl-de/by-2-0; Saxony-Anhalt: © GeoBasis-DE / LVermGeo ST, dl-de/by-2-0;
- Hamburg: Freie und Hansestadt Hamburg, LGV, dl-de/by-2-0; Hesse: © HVBG, dl-de/zero-2-0; Schleswig-Holstein: © GeoBasis-DE/LVermGeo SH, CC BY 4.0; Mecklenburg-Vorpommern: ©GeoBasis-DE/MV/CC BY 4.0 (Quelle verändert); North Rhine-Westphalia: © Geobasis NRW, dl-de/zero-2-0; Lower Saxony: © GeoBasis-DE/LGLN, Daten geändert, CC BY 4.0; Rhineland-Palatinate: ©GeoBasis-DE / LVermGeoRP, dl-de/by-2-0, [Daten bearbeitet]; Saarland: © GeoBasis DE/LVGL-SL, dl-de/by-2-0, Daten geändert; Baden-Württemberg: Datenquelle: LGL, www.lgl-bw.de, dl-de/by-2-0, Daten geändert; Bremen: © GeoBasis-DE / Landesamt GeoInformation Bremen, CC BY 4.0, Daten geändert; Bavaria: Geobasisdaten: Bayerische Vermessungsverwaltung – www.geodaten.bayern.de (Daten verändert), Lizenz: CC BY 4.0.

Created with freemapper – https://github.com/HenryJobst/freemapper
