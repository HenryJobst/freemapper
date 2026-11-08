# Example projects

[Deutsch](README.md)

Each supported state has an example: an area from which freemapper can derive a lot (an old town, a palace or a
park with relief, dense buildings, stairs, walls and hedges). All examples are sprint maps at 1:4000 (ISSprOM 2019),
made with `freemapper run --pdf --ocad`.

| State | Area | Map objects | OSM candidates | Symbols | Preview |
|---|---|---:|---:|---:|---|
| Brandenburg | [Park Sanssouci, Potsdam](brandenburg-sanssouci/): vineyard terraces, parterres and hedges, palace | 432 | 785 | 18 | <a href="brandenburg-sanssouci/brandenburg-sanssouci.pdf"><img src="brandenburg-sanssouci/preview.jpg" width="220" alt=""></a> |
| Berlin | [Viktoriapark, Kreuzberg](berlin-viktoriapark/): the Kreuzberg hill with its waterfall, 19th-century blocks | 706 | 769 | 19 | <a href="berlin-viktoriapark/berlin-viktoriapark.pdf"><img src="berlin-viktoriapark/preview.jpg" width="220" alt=""></a> |
| Saxony | [Bautzen old town](sachsen-bautzen/): the old town on the rock above the Spree, steep valley sides | 1207 | 1180 | 21 | <a href="sachsen-bautzen/sachsen-bautzen.pdf"><img src="sachsen-bautzen/preview.jpg" width="220" alt=""></a> |
| Saxony-Anhalt | [Schlossberg Quedlinburg](sachsen-anhalt-quedlinburg/): World Heritage old town, castle hill, narrow lanes | 1581 | 953 | 20 | <a href="sachsen-anhalt-quedlinburg/sachsen-anhalt-quedlinburg.pdf"><img src="sachsen-anhalt-quedlinburg/preview.jpg" width="220" alt=""></a> |
| Hamburg | [Treppenviertel Blankenese](hamburg-blankenese/): the steep Elbe bank, stairs and lanes | 944 | 919 | 18 | <a href="hamburg-blankenese/hamburg-blankenese.pdf"><img src="hamburg-blankenese/preview.jpg" width="220" alt=""></a> |
| Hesse | [Marburg old town and castle](hessen-marburg/): the steep upper town with stepped lanes, castle hill | 1514 | 828 | 18 | <a href="hessen-marburg/hessen-marburg.pdf"><img src="hessen-marburg/preview.jpg" width="220" alt=""></a> |

**Map objects** are the reliable objects of the OOM map (ALKIS buildings and areas). **OSM candidates** come as a
separate template (`templates/candidates.omap`); the PDF draws them too. **Symbols** counts the distinct symbols
in both together.

## What an example contains

| File | Content |
|---|---|
| `<name>.omap` | the OOM starter project (OpenOrienteering Mapper) |
| `templates/candidates.omap` | the OSM candidates as a template |
| `ocad/<name>.ocd`, `ocad/<name>-candidates.ocd` | the OCAD starting point (OCAD 12) |
| `<name>.pdf` | the fully generated training map |
| `preview.jpg` | a preview of the training map |
| `freemapper.toml` | the project: area, scale and the fetched data versions |
| `ATTRIBUTION.txt` | the attribution of the data sources |

The raster templates (orthophoto, hillshade, slope, nDSM) and the raw data (`data/`) are not in the repository,
because they take over 20 MB per example. To get them back:

```sh
freemapper run --ocad --pdf examples/sachsen-anhalt-quedlinburg
```

freemapper fetches the data again, computes the templates and keeps the existing maps (`.omap`, `.ocd`) unchanged.
The map then opens in OOM and OCAD with all templates.

## How the areas were chosen

For each state, five to six candidates were made as sprint maps, among them parks, schools, old towns and palace
grounds. The distinct symbols and the objects were counted; the best two or three areas of each state were then
compared on the training map. The areas that show the most won: varied buildings, relief and many OSM candidates.

## Licences

The maps contain derived official geodata and OpenStreetMap data. The attribution is in each `ATTRIBUTION.txt`, in
the PDF's peripheral information and in the map notes:

- OpenStreetMap: © OpenStreetMap contributors, ODbL;
- Brandenburg: © GeoBasis-DE/LGB, dl-de/by-2-0; Berlin: © Geoportal Berlin, dl-de/zero-2-0;
- Saxony: GeoSN, dl-de/by-2-0; Saxony-Anhalt: © GeoBasis-DE / LVermGeo ST, dl-de/by-2-0;
- Hamburg: Freie und Hansestadt Hamburg, LGV, dl-de/by-2-0; Hesse: © HVBG, dl-de/zero-2-0.

Created with freemapper – https://github.com/HenryJobst/freemapper
