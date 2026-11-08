# freemapper – User manual

[Deutsche Version](de.md) · [Glossary](../glossary.md) · [README](../../README.en.md)

This manual is for mappers and coaches who want to make an orienteering map without dealing with
geodata or the command line. It describes the current state, with Brandenburg, Berlin and Saxony.

## Contents

1. [What freemapper does](#1-what-freemapper-does)
2. [What you need](#2-what-you-need)
3. [Installation](#3-installation)
4. [Route A: with an AI assistant](#4-route-a-with-an-ai-assistant)
5. [Route B: without an assistant](#5-route-b-without-an-assistant)
6. [Choosing the map area in the browser](#6-choosing-the-map-area-in-the-browser)
7. [The result](#7-the-result)
8. [What the templates show](#8-what-the-templates-show)
9. [What to check in the terrain](#9-what-to-check-in-the-terrain)
10. [Quick training map (PDF)](#10-quick-training-map-pdf)
11. [OCAD](#11-ocad)
12. [Regions and border areas](#12-regions-and-border-areas)
13. [Working on later](#13-working-on-later)
14. [Messages and what to do](#14-messages-and-what-to-do)
15. [Attribution](#15-attribution)
16. [Help and bug reports](#16-help-and-bug-reports)

## 1. What freemapper does

freemapper fetches the official geodata of an area and builds the **starting point** for a schoolyard
map (1:1000) or sprint map (1:4000). The geodata includes the orthophoto, the terrain model, and the
buildings and land use from the cadastre (ALKIS). Data from OpenStreetMap (OSM) is added.

- **Drawn as finished map objects** is only what is reliable: buildings, and on sprint maps also paved
  areas (streets, squares) and unpaved paths with a known surface.
- **Given as templates** is everything you have to judge yourself: orthophoto, hillshade, slope, height
  above ground (nDSM), contours, fences, hedges, benches, playground equipment and out-of-bounds areas.
- You still draw and check the map in the terrain, in **OpenOrienteering Mapper (OOM)** or **OCAD**.

On request freemapper also writes a **print-ready training map as a PDF** (chapter 10).

## 2. What you need

- A computer with Windows, macOS or Linux. No GIS or other programs are needed.
- [OpenOrienteering Mapper](https://www.openorienteering.org/apps/mapper/) (free) or OCAD 12 or newer.
- An internet connection for the first download.
- Free disk space:
  - The first time, freemapper downloads the OSM extract of the state: about 300 MB for Brandenburg
    and Berlin, about 270 MB for Saxony, about 180 MB for Saxony-Anhalt, about 55 MB for Hamburg, about 350 MB for Hesse, about 160 MB for Schleswig-Holstein (without Hamburg), about 130 MB for Mecklenburg-Vorpommern. It
    is reused by all projects.
  - On top of that, each map usually needs 10–100 MB.
- An area in **Brandenburg, Berlin, Saxony, Saxony-Anhalt, Hamburg, Hesse, Schleswig-Holstein or Mecklenburg-Vorpommern**. More states will follow.
- Optionally an AI assistant such as Claude, so you do not have to type commands.

## 3. Installation

freemapper is a single program without an installer.

1. On the [release page](https://github.com/HenryJobst/freemapper/releases/latest), download the archive
   for your system:
   - **Windows:** `freemapper-…-windows-x86_64.zip`
   - **Mac with Apple chip** (M1 and newer): `freemapper-…-macos-arm64.tar.gz`
   - **Mac with Intel processor:** `freemapper-…-macos-x86_64.tar.gz`
   - **Linux:** `freemapper-…-linux-x86_64.tar.gz`
2. Unpack the archive with a double click. Put the folder in a fixed place, e.g. `Documents/freemapper`.
3. On the first start the system warns you, because the program is not signed:
   - **Windows:** "Windows protected your PC" → *More info* → *Run anyway*.
   - **macOS:** "cannot be opened …" → *System Settings › Privacy & Security* → *Open Anyway*.

Note the **full path** to the program, e.g. `C:\Users\Anna\Documents\freemapper\freemapper.exe` or
`/Users/anna/Documents/freemapper/freemapper`. The assistant needs it.

## 4. Route A: with an AI assistant

The assistant operates freemapper for you. For this freemapper is an "MCP server", a tool that AI
assistants can use.

### Set it up once

- **Claude Desktop:**
  1. Open *Settings › Developer › Edit Config*.
  2. Add this to the file `claude_desktop_config.json` (adjust the path; on Windows use double
     backslashes `\\`):

     ```json
     {
       "mcpServers": {
         "freemapper": { "command": "/Users/anna/Documents/freemapper/freemapper", "args": ["mcp"] }
       }
     }
     ```

  3. Restart Claude Desktop.
- **Claude Code:** in a terminal, enter `claude mcp add freemapper -- /path/to/freemapper mcp`.
- **Other assistants:** any MCP-capable assistant takes the same entry: the program with the argument
  `mcp`.

### Order a map

Simply tell the assistant what you need, for example:

> Use freemapper to make a schoolyard map of the Evangelisches Kreuzgymnasium in Dresden.
> Save the project in Documents/O-maps/kreuzgymnasium.

> Make me a sprint map of Strausberg's old town, with an OCAD file and a training map as PDF.

The assistant then goes through these steps:

1. It searches the place and shows you the results.
2. It creates the project.
3. It tells you how much will be downloaded.
4. It starts downloading and computing. The first time this takes a few minutes, later usually less
   than a minute.
5. It shows you previews of the templates.

If no search result fits, for example because the school grounds are two separate parts in OSM, ask
the assistant to **open the area selection in the browser**. There you draw the area yourself
(chapter 6).

## 5. Route B: without an assistant

It does not work entirely without the command line, but two commands are enough. Open a terminal
(on Windows, PowerShell) in the folder with freemapper.

1. Choose the area in the browser. This also creates the project folder:

   ```sh
   ./freemapper select kreuzgymnasium
   ```

   On Windows: `.\freemapper.exe select kreuzgymnasium`.
2. Download everything, compute it and write the map:

   ```sh
   ./freemapper run kreuzgymnasium
   ```

   `--ocad` also writes the OCAD file, `--pdf` the training map.

If you know the OSM object of the grounds, you can skip the browser:

```sh
./freemapper init kreuzgymnasium --osm way/23838018 --title "Evangelisches Kreuzgymnasium"
./freemapper run kreuzgymnasium
```

## 6. Choosing the map area in the browser

![Area selection in the browser](images/area-selector.jpg)

*The area selection with the Kreuzgymnasium Dresden (German interface). Map data © OpenStreetMap
contributors.*

- **Search a place:** type a name, e.g. "Kreuzgymnasium Dresden", and pick a result with an area.
- **Draw it yourself:**
  - *Rectangle:* click two corners.
  - *Polygon:* click the corners one by one. To close it, click the first corner or double-click.
  - Esc cancels, Backspace removes the last corner.
- **Background:** at the top right, choose OpenStreetMap or the orthophoto (LGB for Brandenburg and
  Berlin, GeoSN for Saxony).
- **On the right** you see:
  - the size with buffer;
  - the data tiles that will be downloaded (thin lines on the map);
  - the estimated download.
- Set **map kind, scale and title**, then click *Save* and *Done*.

The page runs only on your computer. With its buffer, an area may be at most 4 km × 4 km.

## 7. The result

Among other files, the project folder then holds:

| File | What it is |
|---|---|
| `<name>.omap` | the map for OpenOrienteering Mapper, georeferenced, with buildings and the peripheral information |
| `templates/` | the templates: orthophoto, hillshade, slope, nDSM and the candidates map |
| `ocad/` | with `--ocad`: the OCAD map and its templates |
| `<name>.pdf` | with `--pdf`: the training map |
| `ATTRIBUTION.txt` | the attribution of the data sources |
| `freemapper.toml` | the project's settings and data currency |

Open the `.omap` file in OOM. The templates are already linked; show and hide them in the *Templates*
menu. The **candidates** (`templates/candidates.omap`) are a separate map on top: fences, hedges,
benches, contours and out-of-bounds areas. Check them and take over what is right: trace them, or open
the candidates map and copy and paste the objects. Both maps have the same georeferencing.

Below the map is the **peripheral information**: title, scale, contour interval, scale bar, north
lines and the attribution. Keep the attribution on printed maps (chapter 15).

The example projects in the repository show what the result looks like, one per state:
[`examples/`](../../examples/README.en.md). Each has the OOM map, the OCAD starting point and the PDF training map.

## 8. What the templates show

The examples show the Kreuzgymnasium Dresden. Orthophoto, terrain and surface model: Quelle: GeoSN,
dl-de/by-2-0, Daten geändert.

| Template | Picture |
|---|---|
| **Orthophoto:** top view, 20 cm per pixel. Good for paths, squares and vegetation, but trees hide the ground. | ![Orthophoto](images/orthophoto.jpg) |
| **nDSM (height above ground):** yellow 0.3–1 m (walls, benches, cars), orange 1–2.5 m (hedges, fences, playground equipment), light green 2.5–8 m (bushes, sheds), dark green above 8 m (trees, houses). Shows what stands, also under trees. | ![nDSM on the orthophoto](images/ndsm.jpg) |
| **Hillshade:** the terrain as if lit from the north-west. Shows banks, hollows and earth walls. | ![Hillshade](images/hillshade.jpg) |
| **Slope:** the darker, the steeper. Helps with banks and stairs. | ![Slope](images/slope.jpg) |

The candidates map also holds the **contours**: every 1 m on schoolyard maps, every 2 m on sprint maps,
every fifth one an index contour. They are computed from the terrain model and smoothed. For the map
they are a draft, not a finished result.

## 9. What to check in the terrain

freemapper saves you drawing the base, not the walk through the terrain. Check above all:

- **Buildings:**
  - Are outline and passages right?
  - Are new buildings missing, or demolished ones still there?
  - Each building's data currency is in its object tags.
- **Paved areas** (sprint): in Berlin and Saxony, street areas include the grass and tree strips at
  their edges. Squares there are only candidates.
- **Fences, walls, hedges:** they come from OSM and the nDSM. Can they be crossed? Are there gates?
- **Out-of-bounds areas:** private grounds, railway land, and on schoolyard maps everything outside the
  school grounds.
- **Furniture and playground equipment:** it comes from OSM and is often incomplete.
- **Contours:** small banks and steps show better in the hillshade or slope.

## 10. Quick training map (PDF)

With `--pdf` (or "with a training map" to the assistant), freemapper writes `<name>.pdf`: a fully
drawn map in A4 or A3, ready to print.

![Training map as PDF](images/training-map.jpg)

*The training map of the Kreuzgymnasium. Quelle: GeoSN, dl-de/by-2-0, Daten geändert; © OpenStreetMap
contributors.*

Unlike the starting point, **every candidate becomes a map object** here, also the uncertain ones. The
map is good for a quick training session. Still check it in the terrain first.

## 11. OCAD

With `--ocad` (or "with an OCAD file"), freemapper writes the folder `ocad/`:

- `<name>.ocd`: the map in OCAD 12 format, with georeferencing, buildings and the templates as
  background maps;
- `<name>-candidates.ocd`: the candidates as a separate map;
- the template images with world files.

The symbols in the OCAD file are placeholders with the right number and colour, but simple graphics.
Load the official symbol set in OCAD:

1. Choose *Symbol › Load Colors and Symbols from …*.
2. Pick the set: ISSprOM 2019 for sprint maps, the IOF school set 2019 for schoolyard maps.

OCAD then replaces the placeholders by number. The OCAD output has not been checked in every OCAD
version yet; feedback is welcome (chapter 16).

## 12. Regions and border areas

| State | Data from | Particulars |
|---|---|---|
| Brandenburg | LGB | orthophoto, terrain and nDSM in 1 km tiles; the nDSM is a laser nDSM at 20 cm |
| Berlin | LGB (rasters), Geoportal Berlin (ALKIS) | the nDSM is computed from the image-based surface model: a bit softer, without thin objects such as lamp posts, but with parked cars |
| Saxony | GeoSN | orthophoto from the map service; the nDSM has only 1 m resolution, so hedges and narrow walls often show faintly; underground structures cannot be filtered out |
| Saxony-Anhalt | LVermGeo | the map is in UTM zone 32 (EPSG:25832), like all of the state's data; orthophoto from the map service, terrain and surface model at 1 m; the nDSM is as coarse as in Saxony |
| Hamburg | LGV | zone 32 like Saxony-Anhalt; orthophoto from 2026, but terrain and surface model from 2022: buildings of the last years are missing from the nDSM |
| Hesse | HVBG | zone 32; orthophoto flown in summer (2023 south-west, 2024 north, 2025 south); nDSM = DOM1 − DGM1 at 1 m from the laser scan (2019–2025 by region). In many areas the scan was flown in winter, without leaves, so the crowns of deciduous trees can come out lower than in summer. The scan can be from another year than the orthophoto (Frankfurt: scan 28.02.2023, orthophoto 19.08.2025). The height models come from the Downloadcenter, one zip per municipality; freemapper reads only the needed 1 km tiles. Squares without `bez` only as candidates; street areas include verges (no split); school grounds are called "Öffentliche Zwecke" |
| Schleswig-Holstein | LVermGeo SH | zone 32; the state was flown in two halves: 2024 (May/September, leaf-on) and 2025 (March–June: March/April leaf-off, May/June leaf-on); Norderstedt, for example, was flown leaf-off on 17.03.2025. The map service has no time choice. Terrain: DGM1 at 1 m from the laser scan (vintage 2005–2025 by tile; Norderstedt 10.03.2022). Surface model: image-based bDOM at 0.2 m (only the 2023 and 2024 vintages); the nDSM is bDOM − DGM1. freemapper downloads whole tiles of 105 MB for it (about 90 s each, no partial downloads), reduces them to 1 m and keeps only that copy. In the 2025 half the bDOM is two years older than the orthophoto: new buildings may be missing in the nDSM (freemapper warns). The Downloadportal sends an incomplete certificate chain; freemapper ships the missing D-TRUST certificate and uses it for that one server only. Squares without `bez` only as candidates; street areas partly without a verge split; school grounds are called "Öffentliche Zwecke"; the LGB's ALKIS spellings apply. At the Hamburg border freemapper reads the ALKIS and OSM extract of both sides; the rasters come from the state where the centre of the map area lies, and the Schleswig-Holstein tiles reach into Hamburg |
| Mecklenburg-Vorpommern | LAiV M-V | zone 33 (EPSG:25833) like Brandenburg. The orthophoto comes from the WCS service "DOP" (the map service carries a watermark) as JPEG and is a 256-colour palette image: slightly grainy, but good enough for tracing. Flown 2021–2025, March to October, mostly leaf-on (about 23 % in March/April, then leaf-off); Rostock, for example, on 14.08.2024. Terrain and surface: DGM1 and DOM1 at 1 m from the WCS; the nDSM is DOM1 − DGM1. The two models can come from different years (Rostock: DGM1 March 2024, DOM1 August 2024). The height models carry no NoData: freemapper keeps only the cells of the state's 2 km tiles. Areas beyond them (Brandenburg south of the tile edge, open sea outside the tiles) stay empty; sea and Bodden inside the tiles are 0 m. ALKIS comes from the LGB's OGC API (as in Brandenburg), with the WFS as fallback; the LGB's ALKIS spellings apply, school parcels are called "Bildung und Wissenschaft". At the Brandenburg border freemapper reads the ALKIS and OSM extract of both sides; the rasters come from the state where the centre of the map area lies |

**Coordinate system:** Brandenburg, Berlin, Saxony and Mecklenburg-Vorpommern deliver in UTM zone 33, Saxony-Anhalt, Hamburg,
Hesse and Schleswig-Holstein in zone 32. Each map gets the zone of the state holding its centre; OOM and OCAD show it in the
georeferencing. If you move a map area across the zone line or into another state, freemapper fetches
everything again.

**At a state border** freemapper queries the cadastre of both states and reads both OSM extracts. The
orthophoto and terrain come from the state holding the centre of the map area. If the area reaches far
across the border, the templates stay empty on the other side. freemapper tells you so.

## 13. Working on later

- **Your map is safe:** running `run` again (or "please fetch again") leaves an existing map unchanged.
  It only updates the templates and candidates. If buildings are missing from the map, freemapper
  tells you.
- **Write it again:** `--force` replaces the map. The old one is kept as `.omap.bak`.
- **Change the area:** run `select` on the existing folder, then `run` again.
- **Without internet:** `--offline` works with the data downloaded before.
- **Data currency:** freemapper only downloads what was republished. If a provider changes a tile, you
  get a "changed upstream" warning.

## 14. Messages and what to do

| Message (start) | Meaning | What to do |
|---|---|---|
| `the map area lies outside the supported states` | The area is not in Brandenburg, Berlin, Saxony, Saxony-Anhalt, Hamburg, Hesse, Schleswig-Holstein or Mecklenburg-Vorpommern. | Choose another area; more states will follow. |
| `map area plus buffer is … more than 4000 m per side` | The area is too large. | Draw it smaller or split it into several maps. |
| `no area found for …` | The place was not found, or its grounds are several parts in OSM. | Spell it differently, or draw the area in the browser. |
| `MultiPolygon with … parts` | The chosen grounds consist of several parts. | Draw one connected area in the browser. |
| `warning: no ALKIS buildings found in the map area` | There are no buildings in the area, e.g. in a forest. | Nothing, if that is right. Otherwise fetch again later. |
| `the map area reaches beyond the state border` | Part of the area lies in the neighbouring state. | The templates stay empty there. |
| `download of … failed` or `invalid ALKIS data` | A server was unreachable or sent nonsense. | Try again later. Data fetched before is kept. |
| `… refused the request (HTTP 400)` | A service refuses the request. | Try again. If it persists, report a bug. |
| `… ignored the byte range` | The server (height models of Hamburg or Hesse) sends the whole archive instead of the tile; freemapper stops rather than download over 1 GB. | Try again later, otherwise report a bug. |
| `download of … is larger than … MB; stopped` | A server (the OSM extract, the Downloadportal of Schleswig-Holstein) sends far more than the file can be; freemapper stops and removes the partial download. | Try again later, otherwise report a bug. |
| `no tile of the … covers the map area` | Hamburg: the statewide archive has no tile for the area; it was probably renamed. Hesse: the Downloadcenter may have changed. Schleswig-Holstein and Mecklenburg-Vorpommern: the Downloadportal may have changed. | Report a bug. |
| `warning: the … has no tile(s) … inside Hamburg` | Single tiles are missing from the archive. | The templates stay empty there. |
| `warning: the … has no tile(s) … inside Hesse` | Single tiles are missing from the municipality's Downloadcenter archive. | The templates stay empty there. |
| `warning: the Downloadcenter has no file for "…"` | The Downloadcenter has no file for the place (e.g. an unincorporated area). The tiles it names after `;` (if any) stayed empty because no other municipality's file holds them. | Tiles taken from neighbouring municipalities need no action; the named tiles stay empty. |
| `warning: the metadata list has no readable Bildflugdatum for tile …` | The HVBG metadata list has no flight date for these tiles. | Only the date in the project record is missing. |
| `warning: the new HVBG metadata list … could not be read …` | The newly published list could not be read; the older kept list is used. | Update freemapper if it persists. |
| `warning: the … has no tile(s) … inside Schleswig-Holstein` | Single tiles are missing from the Downloadportal. | The templates stay empty there. |
| `the Downloadportal lists at most … tiles per request` | (Schleswig-Holstein, Mecklenburg-Vorpommern) The area needs more tiles than the Downloadportal lists per request. | Choose a smaller area. |
| `the Downloadportal answered with an error page` | The Downloadportal (Schleswig-Holstein) sends an error page instead of the tile. | Try again later; freemapper retries the tile once itself. |
| `warning: DGM1 tile … is not in the INSPIRE ATOM directory` / `… does not read (…); used its XYZ from the Downloadportal` | The GeoTIFF of the terrain tile is missing in the INSPIRE ATOM directory or does not read (error page, another grid, server error); freemapper took the same tile as XYZ from the Downloadportal. If that fails too, the error names both reasons. | Nothing; the heights are the same. |
| `the Downloadportal's tile list … could not be read` | (Schleswig-Holstein, Mecklenburg-Vorpommern) The tile list could not be read; the one from an earlier run is used. | Nothing, as long as the data is the same. |
| `warning: the surface model (bDOM …) is older than the orthophoto (…)` | The bDOM is at least a year older than the orthophoto (in Schleswig-Holstein from the 2025 half on). | Check new buildings in the nDSM by hand if needed. |
| `warning: surface model (LVermGeo SH bDOM) unavailable` | The bDOM could not be fetched; an error there is only a warning. With `…; kept the surface model fetched on …` the surface model of an earlier fetch of the same area is kept and the nDSM still comes from it (also for the HVBG DOM1). | With `no nDSM template` the nDSM template is missing; try again later. |
| `downloading bDOM tile … (105 MB, … of …)` | Progress message: freemapper downloads a whole tile, which takes about 90 s. | Wait. |
| `geodaten.schleswig-holstein.de answered with a redirect (HTTP …) to …` | The Downloadportal forwards to another address. freemapper does not follow it, because it checks this server against its own certificates only. | Report a bug. |
| `… certificate …` (server `geodaten.schleswig-holstein.de`) | The certificate check for the Downloadportal failed, for example because its certificate authority changed. | Update freemapper, otherwise report a bug. |
| `warning: the … has no tile(s) … inside Mecklenburg-Vorpommern` | Single tiles are missing from the state survey's Downloadportal (only inside the state). | The templates stay empty there. |
| `… cells inside the Mecklenburg-Vorpommern tiles have no height` | Single cells inside the state tiles have no value in the height model. | The templates stay empty there. |
| `… no date for the centre tile …` | The Downloadportal names no date for the tile at the centre of the map area (border map, centre beyond the state tiles). | Only the date in the project record is missing. |
| `no TIFF in the WCS answer: …` | The WCS service of the Mecklenburg-Vorpommern height models sent an error message instead of the image; its text follows. | Try again later, report a bug if it persists. |
| `the WCS answered with an error: …` / `not a JPEG (…): …` | The WCS service of the Mecklenburg-Vorpommern orthophoto sent an error message or something other than a JPEG instead of the image; the cleaned start of the answer follows. | Try again later, report a bug if it persists. |
| `…; the stored model keeps its recorded date` | Offline: the tile list could not be read; the stored height model is used and keeps the date of its record. | Fetch once online. |
| `warning: … unavailable (…); no nDSM template` (LAiV M-V DOM1) | The DOM1 could not be fetched; an error there is only a warning. | The nDSM template is missing; try again later. |
| `warning: the map area now lies in another state's data` | The area now lies with another provider. | Nothing; freemapper fetches everything again. |
| `offline and not cached` / `no OSM extract cached` | Data is missing for `--offline`. | Fetch once with internet. |
| `no tiles fetched yet` / `the map area needs tiles … that were not fetched` | Nothing was fetched yet, or the area changed. | Run `run` or `fetch`. |
| `map already exists (it may contain your work)` | A map exists already. | Use `--force` only if it should be replaced. |
| `warning: … changed upstream` | The provider published new data. | Nothing; the templates are current now. |

## 15. Attribution

The geodata is under open licences, but they require attribution. freemapper writes it into the
peripheral information, the map notes and `ATTRIBUTION.txt`, for example:

- `© OpenStreetMap contributors, ODbL`
- `© GeoBasis-DE/LGB <year>, dl-de/by-2-0, Daten geändert` (Brandenburg; in Berlin with
  `© Geoportal Berlin, dl-de/zero-2-0` added)
- `Quelle: GeoSN, dl-de/by-2-0, Daten geändert` (Saxony)
- `© GeoBasis-DE / LVermGeo ST <year>, dl-de/by-2-0, Daten geändert` (Saxony-Anhalt)
- `Freie und Hansestadt Hamburg, Landesbetrieb Geoinformation und Vermessung (LGV) <year>, dl-de/by-2-0, Daten geändert` (Hamburg)
- `Geobasisdaten © Hessische Verwaltung für Bodenmanagement und Geoinformation (HVBG) <year>, dl-de/zero-2-0, verändert` (Hesse)
- `© GeoBasis-DE/LVermGeo SH/CC BY 4.0 (Quelle verändert)` and the licence link https://creativecommons.org/licenses/by/4.0/ (Schleswig-Holstein)
- `©GeoBasis-DE/MV/CC BY 4.0 (Quelle verändert)` and the licence link https://creativecommons.org/licenses/by/4.0/ (Mecklenburg-Vorpommern)
- `Created with freemapper – https://github.com/HenryJobst/freemapper`

Keep these lines on printed and published maps.

## 16. Help and bug reports

Please report questions, bugs and wishes as an
[issue on GitHub](https://github.com/HenryJobst/freemapper/issues). It helps to include:

- the message;
- the place (name or coordinates);
- the project's `freemapper.toml`.
