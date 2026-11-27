# Background of the examples

[Deutsch](hintergrund.md) · [Examples overview](README.en.md)

## How the examples were made

Each supported state has an example: an area from which freemapper can derive a lot (an old town, a palace or a
park with relief, dense buildings, stairs, walls and hedges). All examples are sprint maps at 1:4000 (ISSprOM 2019),
made with `freemapper run --pdf --pdf-candidates --ocad --training` and the default content profile `balanced`;
Sanssouci, Viktoriapark, Monschau, Cochem and Saarlouis also with `--content-set vegetation=candidates`. The preview
and the PDF of each example show the training map, ready to print and run on. Next to it is the check print, which
draws every candidate true to position and so shows what freemapper can derive. The base map alone shows less; the
[comparison page](compare.en.md) shows the same area in the three content profiles.

## Numbers and sources

**Map objects** are the objects of the OOM map. With `balanced` these are the objects of the cadastre, the registers
and the street survey (buildings, streets with curbs, water, parks, forest, canopies and passages), plus walls,
fences, hedges, trees, stairs and paths from OSM as well, and the contours. **Candidates** are what stays uncertain,
mostly out-of-bounds areas (520) and control sites (531), plus banks, cliffs and knolls from the terrain model (group
`landforms`). The light level adds what it takes off the base map (ADR 0157): hidden lines and objects below the
minimum size. The candidates come as seven candidate layers by kind, `templates/candidates-<layer>.omap`
(ADR 0156), hidden when the map opens. Every layer holds the outline of the map area; it counts once. The PDF
`<name>-candidates.pdf` draws them too; `<name>.pdf` from `--pdf` shows only the map objects. Both come from OSM, from
ALKIS and from the terrain model, in Berlin also from the Geoportal Berlin, and in Potsdam from the city map tree points (Bäume der Stadtkarte).
**Symbols** counts the distinct symbols in
both together. Contours run through ways and paved areas,
as ISSprOM 101 draws them, and stop only at stairs, small landforms and below cliffs; north lines stop at ways, so
they count in pieces.

The group `vegetation` is off in every profile, because the point clouds are large. Five examples switch it on as
candidates: Sanssouci, Viktoriapark, Monschau, Cochem and Saarlouis. There the check print shows what the laser
points say about the vegetation: runnable forest (405) under the tree crowns, open land (401) and the green shades
406, 408 and 410. Sanssouci has 173 forest areas, 75 areas of open land and 205 green areas, Monschau 182, 196 and
638, Cochem 126, 73 and 404, Saarlouis 201, 70 and 125, Viktoriapark 63, 50 and 107. The park turns white where trees
stand and stays yellow where there is lawn. Every flight was made without leaves; the green shades follow how much
stands between 0.8 and 3 m (ADR 0138). Areas that shall not be entered (520) hold no vegetation: the areas end at them, as they do at
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
107 fences (518), 133 hedges (410.1) and 125 embankment tops (104), with `balanced` on the map. Where one of these lines
lies on another symbol, the light level moves it, or the stretch that lies there, into the candidate layers; 224 wall
lines, 71 fences, 119 hedges and 111 embankment tops stay on the map. ALKIS often records both edges of a wall.
Where they face each other and lie closer than 0.55 mm on paper, freemapper draws one
line in the middle; Sanssouci has 14 of these. Thicker walls and edges that face the same way stay double; delete the
second line in OOM. Where OSM has the same wall, hedge or fence, only the ALKIS line stays, and it names the OSM line
in `freemapper:also`. Where a line with another symbol runs parallel and too close, such as an OSM retaining wall on
a terrace wall or a fence beside a wall, the weaker line loses that stretch. Wall stretches along a building outline
and trees within 1 m of a hedge, wall or fence are dropped. In Sanssouci this leaves only a few parallel line pairs
closer than 1 m. The Potsdam city map tree points are a tree cadastre as in Berlin. Of them, 775 stay points: 660 are
on the map, and 115 lie in out-of-bounds areas (520) and stay candidates. The others lie in tree group areas or are
dropped by the thinning. OSM trees closer than 2 m to a cadastre tree are dropped.

In Berlin, freemapper loads more layers from the Geoportal Berlin. Stairs, walls, fences, embankment edges, monuments
and passages go onto the map with `balanced`, and so do the trees from the tree cadastre, 1926 of them in the
Viktoriapark. The Berlin 1:5000 map (K5) comes as the template `templates/k5.png`.

Berlin projects also get the street survey (Straßenbefahrung, recorded 2014 to 2016), the registers of street
lamps, ornamental fountains and drinking fountains, and the building heights from the Umweltatlas. With `balanced`
their objects are on the map. In the Viktoriapark these are 76 curbs (501.1), 75 surveyed paved areas (501.5) and
five small green strips along the curb that freemapper cuts out of one ALKIS street area. The areas are nearly all
sidewalks and the paved paths in the park. Traffic barriers (518), walls (515), bus shelters (522), and street pumps
and ornamental fountains (311) come as well. Advertising columns and street lamps stay candidates as control sites
(531). Only the training map draws the 67 surveyed carriageways, as paved areas with heavy traffic (501.11); the
template `templates/candidates-paths.omap` holds them as hidden candidates. Where a survey barrier
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
in the Viktoriapark 5. ALKIS woods become forest (405), in Monschau 164 areas. Under everything lies the walkable
area, one brown area over the whole sheet (symbol 990, map part `base`). It fills whatever nothing else shows. The PDF with candidates leaves out points inside a 520 candidate; in the
template they carry `freemapper:hidden=520`.

In the Viktoriapark the stairs on the plinth of the Nationaldenkmal lead up to the monument. Wide stairs such as the
stands at the stadium come as parallel 532.9 lines tagged `freemapper:size=wide`. Fences and walls that both ALKIS and
OSM have appear once on the map, as the ALKIS line, which names the OSM line in the tag `freemapper:also`.

## Forest maps

The areas are about 1.2 × 0.9 km and were chosen by relief and path network. Made with
`freemapper init <folder> --geojson <rectangle> --kind forest --scale 10000 --title "<title>"` and
`run --pdf-candidates --ocad --training`. Berlin and Bremen take their outline from OSM (`--osm relation/4513841` for the
Dammheide, `--osm way/664446849` for the Bürgerpark). In Brandenburg, North Rhine-Westphalia, Saarland and
Bavaria `--content-set vegetation=candidates` was added: the vegetation from laser point clouds is in the
candidates map there. Rhineland-Palatinate lacks it, because the state office's Metalink file names other sizes
and checksums for the tiles near Dahn than the files on the server have (November 2026). On the Schmausenbuck
freemapper loads four tiles for it, 1.2 GB together. The flight of 20 December 2021 was made without leaves. The
strip under the power lines comes out mostly as open land, its edges as 406. 29 vegetation candidates are left:
19 of open land (401), 4 areas each of 406 and 410 and 2 of 405.

The paths come from OSM and go onto the map in the path classes 502 to 507; the forest is white paper. Settlements
and gardens are olive green (520, not to be entered). Rocks, earth banks and knolls from the terrain model are
candidates; cliffs from OSM (`natural=cliff`) are on the map, generalized and smoothed by the ISOM (ADR 0126): at
the Bastei 242 of 2144 cliffs stay with tags, 23 as top lines (201.3) and 26 as rock pillars and fins (206); the
others are candidates, which the check print draws all of; the training map (`--training`) generalizes them more. In the flat areas in Berlin and Bremen the contours lie every 2.5 m, in the
steep Bode valley and at the Bastei every 10 m (`--contour-interval 10`, more than the 5 m of the ISOM, so that the
slopes stay legible), elsewhere every 5 m; `freemapper.toml` names it as `[contours] interval_m`.

## What an example contains

| File | Content |
|---|---|
| `<name>.omap` | the OOM starter project (OpenOrienteering Mapper) |
| `templates/candidates-<layer>.omap` | the candidates in seven layers: `terrain`, `water`, `vegetation`, `paths`, `barriers`, `features`, `out-of-bounds` |
| `ocad/<name>.ocd`, `ocad/<name>-candidates-<layer>.ocd` | the OCAD starting point (OCAD 12), the candidates as seven background maps |
| `<name>-candidates.pdf` | the check print: map objects and every candidate, true to position, without a scale bar |
| `<name>-training.omap`, `<name>-training.pdf` | the training map: map and candidates, cleaned up for printing and running |
| `preview.webp` | preview of the training map |
| `preview-thumb.webp` | a small copy of the preview for this overview, made with `scripts/thumbnails.sh` |
| `<name>.pdf`, `<name>-safe.pdf`, `<name>-full.pdf`, `preview-safe.webp`, `preview-balanced.webp`, `preview-full.webp`, `preview-candidates.webp` | only in Cochem: the same area in the three content profiles, without candidates, the previews and a preview of the check print, see the [comparison page](compare.en.md) |
| `freemapper.toml` | the project: area, scale and the fetched data versions |
| `ATTRIBUTION.txt` | the attribution of the data sources |
| `permits.md`, `permits.json` | the permit notes, in German: administrative units, protected areas, operators, classified roads and, where the state has services of its own, Bezirk, forest administration, water protection areas and forest functions (made with `freemapper permits` on 14 November 2026; a hint, not legal advice) |

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
compared on the PDF with every candidate. The areas that show the most won: varied buildings, relief and many OSM candidates.

In Lower Saxony the Großer Garten won, although before the content profiles its map had the fewest map objects. Its
hedges, parterres, rows of trees and fountains gave the most candidates per map object. With `balanced` they are on
the map, among them 301 hedges (410.1), 1124 trees (418) and 17 fountains (311). The light level moves hedges that
lie on a path into the vegetation layer, whole or as a piece (189 objects), and 545 candidates are left.

In Rhineland-Palatinate eight rectangles ran: Koblenz at the Deutsches Eck, Trier, Speyer, Mainz, Bad Kreuznach and
Cochem. Bad Kreuznach and Cochem ran twice, because the first rectangle in Bad Kreuznach missed the Nahe and the one
in Cochem missed the Reichsburg. Speyer and Cochem both reached 24 symbols. Cochem won because only this area has
strong relief: dense contours on the castle hill and the valley sides, plus the Moselle and the narrow old town.
Speyer is flat. Koblenz shows the Rhine and the Moselle at the Deutsches Eck, but it reached only 23 symbols and has
little relief.

In Saarland two rectangles ran, both in Saarlouis. The second lies 180 m further west and takes in the Vauban
island in the old arm of the Saar and the ramparts in the Stadtgarten. Saarlouis was the first example made with
the content profile `balanced`, which draws the objects of the cadastre and the registers straight onto the map. Its
map has 31 symbols, among them 21 stairs from OpenStreetMap, and 721 candidates are left, most of them vegetation.
Since the land uses of Saarland are read in both spellings, 118 street areas and the Saar from ALKIS are on the
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
the Schmausenbuck; half of the southern one was the zoo. The landscape model of the agency (ATKIS Basis-DLM)
adds 8 candidates in Regensburg: 3 fountains, a monument, 2 named trees and 2 watercourses; 2 more fountains stand
in basins OpenStreetMap draws already. On the Schmausenbuck
it adds 14: fences and 6 paths OpenStreetMap lacks. Joined where they meet, the paths of the model are 61 lines,
and OpenStreetMap already has the other 55.

## Licences

The maps contain derived official geodata and OpenStreetMap data. The attribution is in each `ATTRIBUTION.txt`, in
the PDFs' peripheral information and in the map notes:

- OpenStreetMap: © OpenStreetMap contributors, ODbL;
- Brandenburg: © GeoBasis-DE/LGB, dl-de/by-2-0; Potsdam: Landeshauptstadt Potsdam / Bäume der Stadtkarte, dl-de/zero-2-0; Berlin: © Geoportal Berlin, dl-de/zero-2-0;
- Saxony: GeoSN, dl-de/by-2-0; Saxony-Anhalt: © GeoBasis-DE / LVermGeo ST, dl-de/by-2-0;
- Hamburg: Freie und Hansestadt Hamburg, LGV, dl-de/by-2-0; Hesse: © HVBG, dl-de/zero-2-0; Schleswig-Holstein: © GeoBasis-DE/LVermGeo SH, CC BY 4.0; Mecklenburg-Vorpommern: ©GeoBasis-DE/MV/CC BY 4.0 (Quelle verändert); North Rhine-Westphalia: © Geobasis NRW, dl-de/zero-2-0; Lower Saxony: © GeoBasis-DE/LGLN, Daten geändert, CC BY 4.0; Rhineland-Palatinate: ©GeoBasis-DE / LVermGeoRP, dl-de/by-2-0, [Daten bearbeitet]; Saarland: © GeoBasis DE/LVGL-SL, dl-de/by-2-0, Daten geändert; Baden-Württemberg: Datenquelle: LGL, www.lgl-bw.de, dl-de/by-2-0, Daten geändert; Bremen: © GeoBasis-DE / Landesamt GeoInformation Bremen, CC BY 4.0, Daten geändert; Bavaria: Geobasisdaten: Bayerische Vermessungsverwaltung – www.geodaten.bayern.de (Daten verändert), Lizenz: CC BY 4.0.

Created with freemapper – https://github.com/HenryJobst/freemapper
