# Changes

[Deutsche Version](CHANGELOG.md) · [User manual](docs/manual/en.md)

All notable changes to freemapper are listed here, newest first. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Fixed

- Firm gravel paths are map objects again. `surface=compacted|fine_gravel` and `tracktype=grade2` (paved by ISSprOM 501, "hard gravel") are drawn as a line 501.6 to 501.9 by `width` on the map, no longer only as a candidate; the park maps have their paths back. Where ALKIS or the street survey already draw paving (501.5), the part of the line within it goes (widened by 0.25 mm on paper), and remnants under 1 mm on paper go. Ways without `surface`, unknown surfaces and `tracktype=grade3` to `grade5` stay candidates, and tree groups are still cut at the lines. Viktoriapark: 26 map objects 501.6 instead of 49 candidates, Sanssouci: 192 instead of 218. Amends ADR 0073 (ADR 0079).
- Duplicate areas from several sources are kept once. An OSM passage (`tunnel=building_passage`, `covered`) over a Berlin building part, an ALKIS passage or a canopy, an OSM private strip inside an ALKIS area that shall not be entered, and an OSM bridge over a Berlin bridge deck are dropped when half (520: 80 %) of their area lies under the higher-ranked area, widened by 0.25 mm on paper. The kept object names the other in `freemapper:also`. An OSM memorial within 5 m of a cadastre monument and OSM kerbs and embankments along cadastre ones go too. Viktoriapark: 520 from 117 to 94, 522 from 211 to 192. Lübeck: 520 from 1 125 to 1 112, 522 from 280 to 221 (ADR 0078).
- Control sites in buildings no longer thin their neighbours. The 25 m rule now runs after the sites in buildings are dropped.

### Added

- Berlin control sites on sprint maps. The street survey (bollards, masts, flag poles, benches) and the lighting cadastre (lamps in operation) now give 531 candidates with `freemapper:control` (`bollard`, `mast`, `flagpole`, `bench`, `street_lamp`, plus `advertising_column` for the survey's advertising columns), like the OSM ones. Thinning is shared: a site of one kind within 25 m of another is left out, whichever source it comes from, after duplicates and sites in buildings are gone. A surveyed bollard replaces an OSM one within 2 m. Hydrants and play equipment stay schoolyard-only. A Berlin sprint project fetches seven more WFS requests (33 instead of 26); the disk estimate for the Berlin layers goes from 10.5 to 12 MB per km². Existing maps get this with `run --force` (ADR 0077).
- Drinking fountains are 313 on sprint maps, from the Berlin register and survey and from OSM `amenity=drinking_water`.

## [0.8.0] - 2026-11-09

### Added

- Only reachable passages are drawn. A 522 passage (from OSM, ALKIS or the Berlin building parts) that touches only buildings, 520 areas or water, or joins two walled-in courtyards with no way out, gets `freemapper:hidden=unreachable`: it stays among the candidates and is left out of the training map. A passage counts as reachable when it (or a chain of passages, such as the Lübecker Höfe und Gänge) meets walkable ground that connects to the street network, with 0.5 mm (2 m) of tolerance. Lübeck: 16 of 280 passages hidden, Viktoriapark: 52 of 211 (ADR 0076).
- Memorials from OSM. Memorials, wayside crosses and boundary stones (`historic=memorial|monument|wayside_cross|boundary_stone`) are 526 candidates on sprint maps. Stolpersteine, plaques and memorials that are a bench, a tree or a mural stay out, because 526 is for what is obvious (ADR 0075).
- Control sites for course setters. Bollards, street lamps, waste baskets, bicycle racks and benches from OSM are 531 candidates on sprint maps with the tag `freemapper:control` (the kind of object), never map objects. An object with one of its own kind within 25 m (a row of bollards, a pair of benches, a series of lamps) is left out, and a control site gives way to every other point symbol so that it displaces no trees or fountains. The tag is where a planning tool finds the sites (ADR 0074).
- Access restrictions from OSM. Private and closed ways (`access=private|no`, `foot=private|no`) become 520 candidates on sprint maps, a strip of 2 m (roads 3 m) around the way, and the path line is gone. `foot=yes|permissive|designated` leaves the way as it was. Areas with `access=private|no` (land use, garden, pitch, car park, school grounds) are 520 in their own shape (ADR 0073).
- Building passages from OSM. Ways with `tunnel=building_passage` and covered ways (`covered=yes`, arcades) are now 522 candidates on sprint maps, drawn as a strip of 3 m (or the `width` tag) along the way, with the building left whole. Before, a paved passage was no candidate at all and an unpaved one only a 505.1 line. The rules get a new field, `corridor_m`, for this (ADR 0072). Existing maps get them with `run --force`.
- OSM multipolygon relations are read. Parks, water, woods and buildings that exist only as relations, with the tags on the relation and untagged ways as rings, now reach the rules. freemapper chains the ways into closed rings in either direction and keeps holes, so a courtyard stays out of its building and a pond out of its park. An outer way that repeats the relation's tags is drawn once. In the Viktoriapark example the stored OSM features grow from 752 to 755 and the candidates from 2,093 to 2,094: a pond (301) is new, and two lawns with holes lie under the ALKIS park. Park Sanssouci gains one lawn with 13 holes (1,047 to 1,048 features) once its example is rebuilt. Existing maps get relations with `run --force` (ADR 0070).
- Parks are yellow. Green spaces, parks, settlement green and grassland from ALKIS become open land (401) on the map, woods and copses forest (405), heath 403. Cemeteries, other farmland, moor and swamp are candidates, and so are parks and lawns from OSM where no ALKIS park lies (ADR 0069).
- The walkable area. On the sprint map one area in the lower brown of 501.5 lies under everything over the whole sheet, symbol 990 in its own map part `base`, tag `freemapper:source=base`. It fills every gap that nothing else shows; a sprint map without it is incomplete. Its colour comes after all others, in OOM, OCAD (colour 32, symbol 990.000) and the PDF. Print sprint maps in colour, because in greyscale the brown comes close to paper white (ADR 0069).

### Fixed

- Points inside buildings are dropped. A candidate point on sprint maps (memorial, lamp, waste basket, fountain, spring) whose centre lies in a building footprint can be neither seen nor used and is removed; the dark dots on the church in the Lübeck example were memorials. Trees stay. Lübeck: 526 from 10 to 4, 531 from 195 to 180; Viktoriapark: 531 from 121 to 119 (ADR 0076).
- The street edge no longer makes a hook around each street tree. The survey kerb lines (501.1 candidates) ran around every tree slot cut across the parking strip, so the training map showed a hook at each tree. freemapper now replaces a kerb detour with its chord when it encloses less than 1.5 mm² on paper (24 m² at 1:4000), and drops a closed kerb ring of that size. The rule judges a detour by how far it reaches out of its chord (at least 0.3 of it, at most 10 m wide), so the wider trapezoid slots go as well, and closed kerb rings around lone pits are dropped. A detour must also lie on a straight run, so a spike at a tree and a diagonal across a step of the kerb no longer appear. Curves, larger bays and the map itself stay as they are. In the Viktoriapark the survey kerbs lose 735 of 6,522 vertices and 5 of 81 lines. Existing projects get the cleaner lines with `run --force` (ADR 0071).

### Changed

- The Viktoriapark and Lübeck examples are rebuilt with the new OSM rules. Viktoriapark: map objects 789 to 740 (505.1: 58 to 9, because compacted gravel becomes a candidate), candidates 2,089 to 2,327 (531: 5 to 121, of which 116 control sites; 520: 73 to 117; 522: 179 to 211; 501.6: 45 new, 501.9: 4 new; 526: 1 to 3). Lübeck: map objects 2,294 to 2,281, candidates 1,528 to 1,859 (522: 185 to 280; 531: 0 to 195; 520: 1,090 to 1,125; 526: 0 to 10; 501.6: 13 new; 505.1: 35 to 16). The stored OSM features grow from 755 to 1,371 (Viktoriapark) and from 234 to 927 (Lübeck).
- Compacted and fine gravel (`surface=compacted|fine_gravel`) is firm under ISSprOM 501, so it is no longer an unpaved path on the map but a 501.6 to 501.9 candidate by `width`. Unpaved ways from 3 m wide are 505.2. A track without `surface` follows `tracktype` (ADR 0073).
- Tree group areas (402, 405) no longer cover paths. freemapper also cuts them at paved candidates and at paths (505, 506, 507). The dotted distinct vegetation boundary (416) is left only on the schoolyard map; on the sprint map the white forest shows on yellow or brown. In the Viktoriapark more trees therefore stay points (1,128 instead of 835), and the 49 boundary lines are gone (ADR 0069).
- All eleven example projects are rebuilt, with yellow parks and the walkable area.
- Dense tree groups now become an area the map shows, also in parks. Three or more trees standing closer than 2.5 mm on paper (10 m at 1:4000), or whose crowns almost touch, become forest (405) when their crowns cover at least half of the area, else open land with scattered trees (402). On the white base the forest gets a dotted distinct vegetation boundary (416) so that it shows. On paving it stays 501.3, from 9 trees. Rows of trees along streets and avenues stay points. In Park Sanssouci the tree points drop from 1,654 to 782 (48 areas 402), in Viktoriapark from 1,660 to 942 (43 forests, 9 areas 402; ADR 0066, amendment).
- The training map (PDF) draws the point patterns of areas, such as the white dots of 402. Until now 402 looked like 401 there.
- Overlapping walls, fences and hedges. Where two such lines run parallel and closer than their symbols may come on paper, the weaker one loses that stretch: the worse source, and for one source the weaker symbol (wall before fence before hedge before retaining wall). Wall stretches within 0.5 m of a building outline are dropped, and so are trees within 1 m of one of these lines. In Park Sanssouci the parallel line pairs closer than 1 m drop from 162 to 4 (ADR 0068).
- All eleven example projects are rebuilt with these rules; their map objects stay the same.

- freemapper now writes, copies and removes files in the project folder with the shared crate `freemapper-fs` from freemapper-core 0.2.0 and no longer keeps its own copy. Nothing changes for users, and the messages keep their wording (ADR 0067).

## [0.7.0] - 2026-11-09

### Added

- Canopies and passages in more states. In every state except Saxony and Saxony-Anhalt, ALKIS canopies and carports become map objects (522) on both map kinds, cut at the buildings as in Berlin. Building passages and arcades become 522 candidates, and the building stays whole. freemapper reads them from the buildings it already fetches, so there is no extra download. The data currency is the date of the ALKIS object. An existing map gets the canopies only with `run --force` (ADR 0061).
- Brandenburg: the full ALKIS from the LGB's NAS service. Walls and wall edges, fences, hedges, monuments, fountains, ditches and brooks become candidates, on sprint maps also embankment tops and bridges. The code in the cadastre picks the symbol, and `freemapper:alkis` gives its name, e.g. `Mauerkante, rechts`. The data currency is the date the object version entered the cadastre. Where the two edges of a wall face each other and lie closer than 0.55 mm on paper (2.2 m at 1:4000), freemapper draws one 515 line in the middle with `freemapper:alkis=Mauer (Mauerkanten gepaart)`, and `freemapper:also` names the other edge. Park Sanssouci has 264 wall lines instead of 271. The layers are stored in `data/alkis-nas/`, about 1 MB per km² (ADRs 0062, 0063).
- Potsdam: the trees of the city map (Bäume der Stadtkarte) become 418 candidates, each with its own survey date. The data has no crown size, and stumps are left out. OSM trees within 2 m of a Potsdam tree are dropped. freemapper stops an answer larger than 256 MB while it reads it. The new attribution line `Landeshauptstadt Potsdam / Bäume der Stadtkarte, dl-de/zero-2-0` appears when Potsdam trees are among the candidates (ADR 0065).
- Point symbol generalization. Point symbols keep the gap of 0.15 mm that ISSprOM sets for lines. Three or more trees that crowd each other become an area when at least half of their crowns lie on paving or on open land (401 to 404): paved area with scattered trees (501.3) on paving, and on open land forest (405) when the crowns cover at least half of the group, otherwise open land with scattered trees (402). Elsewhere the trees stay points and are thinned, because a white forest on the white base would hide them. Rows of trees stay single tree symbols. The area carries `freemapper:source=generalization` and lists its trees in `freemapper:source_id` (ADR 0066).

### Changed

- Points too close together are thinned. Where two point symbols would come closer on paper than the norm allows, freemapper keeps the larger one, then the one from the better source, then the one with the larger crown. The kept point names the others in `freemapper:generalized`. This changes the candidates and the training map in every state, Berlin included. The map objects stay the same (ADR 0066).
- Trees and other points in out-of-bounds candidates (520) stay in the candidates with `freemapper:hidden=520`, but the training map leaves them out. On schoolyard maps the PDF therefore shows no point off the school grounds (ADR 0066).
- Duplicates: register before ALKIS before the street survey before OSM now applies in every state, and also to brooks and ditches (305, 306). An OSM line with at least 80 % of its length within 1.5 m of a full ALKIS line with the same symbol is dropped and named in `freemapper:also` (ADR 0063).
- The download estimate and the disk check count the full ALKIS and the city tree cadastres as an optional part (2 and 1 MB per km²). Without room, freemapper leaves them out and keeps the stored ones.
- `run` now reports a kept map without the ALKIS canopies in every state with canopies. The message now reads `The map lacks the ALKIS canopies and carports (522) …`.
- freemapper joins the pieces of a line with a side (earth bank 104, cliff 201, retaining wall 513.2, the right and left edges of the full ALKIS) only in their own direction and never across two sides, so the side of the ticks stays right. In the Viktoriapark two more earth bank lines stay apart (ADR 0063).

### Security

- Cleaning up old `.part` files in the `templates` folder no longer follows a link out of the project and deletes nothing outside it. A `.part` entry that is itself a link is skipped (ADR 0060).

## [0.6.2] - 2026-11-09

### Security

- A project folder from someone else could make freemapper write outside it through links. If `ATTRIBUTION.txt` was a link, every `run` with an existing map overwrote the file it pointed to. With `--force`, the backups `.omap.bak` and `.ocd.bak` copied the project's map into the link's target, for example `~/.zshrc`. The image copies into `ocad/` followed links too, and so did every file written into a `data`, `templates` or `ocad` folder that was a link. freemapper now writes each file under a new name and renames it into place, which replaces a link instead of writing through it. A `data`, `templates` or `ocad` folder that leads out of the project stops the command (`…: a link leads out of the project folder, so freemapper writes nothing there`). Partial files get a fresh name each time and never take over an existing file or link (ADR 0060).
- Copies inside a project could read through links. A template image that was a link to a file elsewhere, such as `~/.ssh/id_rsa`, ended up copied into `ocad/`. freemapper now copies only files of the project itself. The OCAD export leaves such a template out and warns (`warning: …: a link or outside the project folder, so freemapper does not copy it; …`). freemapper checks every single write, not just once per run (ADR 0060).
- A remaining race: someone who can write to a project folder while freemapper runs on it, and who swaps a folder for a link at the right moment, can still redirect a write with a fixed name. Do not run freemapper on project folders that others can write to at the same time (ADR 0060).

### Changed

- Library API (`freemapper` crate): the writers of templates and stored data take the project folder as an extra `root` argument, placed before the output path. This covers `write_gray_png`, `write_ndsm_png`, `build_orthophoto`, `build_orthophoto_from_image`, `wms::fetch_orthophoto`, `wms::fetch_orthophoto_from`, `wms::fetch_k5`, `geosn::fetch_orthophoto` and the `fetch_model` functions of the height model sources. `OcadOutcome` has the new field `warnings`. The command line and MCP are unchanged.

### Fixed

- Windows: the free disk space check before `fetch` now tells volumes apart by their serial number instead of the drive letter. Until now freemapper skipped the check with a warning (`warning: cannot read the free disk space of …`) when Windows could not resolve the path, as on some RAM or virtual drives. A volume mounted into a folder of another drive counted as that other drive, so its own space went unchecked.

## [0.6.1] - 2026-11-09

### Security

- A crafted `freemapper.toml` could make freemapper delete files outside the project. A `berlin_layers` entry named like `../../../x` made `fetch` delete the file `x.geojson` next to the project folder. freemapper now builds the paths of the Berlin layers only from its own layer list. When it loads a project, it drops entries with an unknown name or with `/`, `\`, `..`, a drive letter or control characters, and warns (`warning: freemapper.toml: ignored the Berlin layer entry …`). If the folder `data/berlin` leads out of the project through a link, freemapper writes and deletes nothing there (ADR 0059).

## [0.6.0] - 2026-11-09

### Added

- Berlin: the street survey (Straßenbefahrung) of Geoportal Berlin, recorded from 2014 to 2016. On sprint maps, curbs (501.1), advertising columns (531), green strips (401) and surveyed sidewalks, squares and park paths outside the ALKIS street areas (501.5) become candidates. On both map kinds, stairs, walls, railings, traffic barriers and crash barriers (518), bus shelters and fountains are added. Each surveyed candidate carries its survey date; when that is unreadable, it says 2014-01-01 with `freemapper:currency_guessed=yes`.
- Berlin: street furniture on schoolyard maps. Benches, hydrants, bollards, masts, flag poles and play equipment come from the street survey, lamps (only those in operation) from the lighting cadastre, ornamental and drinking fountains from their registers. Sprint maps get none of it.
- Berlin: a register point replaces OSM and surveyed points of the same kind. A lamp replaces OSM lamps up to 5 m and surveyed masts up to 2 m away, a fountain other fountains up to 3 m away. On schoolyard maps a surveyed hydrant, bench, mast or play item replaces the OSM one up to 2 m away. A fountain of a register or of the street survey inside drawn water is dropped.
- Berlin: buildings carry their footprint date (Grundrissaktualität) from the building heights of 2022 in the Umweltatlas as data currency. A building without a match keeps no date.
- New attribution lines for the street survey, the registers and the building heights, each only when they gave objects. The register line names only the registers whose points are in the map or the candidates.

### Changed

- Berlin: the paved areas (501.5) no longer include the surveyed green strips; freemapper cuts them out. Tree pits stay inside. A green strip is a 401 candidate only where freemapper cut it out or where it lies off the paved areas. A cut area names the strips in `freemapper:cut`, `freemapper:cut_ids` and `freemapper:cut_currency`. An existing map changes only with `run --force`.
- Berlin: short line pieces from ALKIS and the street survey that meet end to end are joined into one line. It names the other pieces in `freemapper:also` and carries the oldest survey date. Lines under 1 mm on paper are dropped after that, now for ALKIS too.
- Berlin: when an object comes from several sources, the register wins over ALKIS, ALKIS over the street survey, and the survey over OSM. A surveyed bus shelter at least half under an ALKIS canopy is dropped. The object that stays names the others in `freemapper:also`, surveyed ones as `geoportal-berlin/<layer>/<gis_id>`. Each Berlin candidate carries the tag `berlin:layer`.
- The download estimate counts 10.5 instead of 4 MB per km² for the Berlin layers (measured at Viktoriapark: 9.8).

## [0.5.0] - 2026-11-09

### Added

- Disk space: before each download freemapper checks the free space of the cache and the project folder, each drive on its own, and keeps 1 GB free on every drive. If not everything fits, it leaves out the surface model, the update of an older OSM extract and, in Berlin, the structures, trees and K5 and warns with the space needed and the space free. Without room for the map data, the terrain model or the orthophoto it stops before it downloads anything. When the disk fills up during a write, freemapper says so plainly, deletes the half-written file and does not retry. The download estimate (MCP tool `estimate_download`, area selection page) shows the free space and marks what does not fit.
- Berlin: more open data from Geoportal Berlin. ALKIS canopies and carports become map objects (522) on both map kinds. freemapper cuts them at buildings and leaves them out when they lie almost wholly inside a building. Candidates now include stairs (532.6 to 532.9) with the walking direction from the terrain model. A stair that does not rise there leads up to an adjoining building or monument, or runs across its short side. Stairs wider than three times 532.9 become parallel lines. The `Freitreppe` lines are step edges, and each flight of parallel edges becomes one stair across them. Further candidates are walls, retaining walls, fences, hedges, building passages and monuments, and on sprint maps also bridges and embankment tops. Street and park trees from the tree cadastre become 417 or 418 by crown size, and OSM trees within 2 m of them are dropped. So are OSM fences, walls and hedges with 80 % of their length within 1.5 m of the same ALKIS line. The Berlin map 1:5000 (K5) comes as the template `templates/k5.png`. An existing map gets the canopies only with `run --force`, and `run` says so.
- `init --osm` takes one part of an OSM area of several separate parts with `--part <n>` (1 is the largest; `part` over MCP). Without `--part` the message lists the parts with their sizes. When Nominatim does not find a way, such as an untagged outer way, the message suggests the relation or `--geojson`.

### Changed

- Allotment gardens (ALKIS land use "Sport-, Freizeit- und Erholungsfläche" with `bez` "Kleingarten") are out-of-bounds candidates (520) in every state. Parks, sports grounds and playgrounds stay open.

### Fixed

- Contour candidates name the terrain model of their state in `freemapper:source`, e.g. `laiv-mv-dgm1` or `hvbg-dgm1`. Before, every state said `lgb-dgm1`.

## [0.4.0] - 2026-11-08

### Added

- Lower Saxony (LGLN): orthophoto from the WMS, terrain and surface models (DGM1/DOM1) from the WCS in 2 km pieces, flight date and data currency from the STAC searches, buildings and land use from ALKIS. The nDSM is DOM1 − DGM1; if the DOM1 is more than two years older than the orthophoto, freemapper warns with both dates. A message says whether the trees were in leaf on the flight. Maps use UTM zone 32. Bremen is not included yet.
- Rhineland-Palatinate (LVermGeo RP): orthophoto from the WMS, with the block's date of creation as the flight date. Terrain and surface models (DGM1/DOM1) come from 1 km laser tiles whose names and dates the `Laserbefliegung` flight index gives; for tiles without a flight year freemapper reads the state's Metalink file. If the server lacks a tile, freemapper reports it as missing and asks again after 30 days. Buildings and land use come from ALKIS. Maps use UTM zone 32. At the Hesse border (Mainz, Wiesbaden) freemapper reads the ALKIS and OSM of both states.
- Water on all maps: rivers, canals, lakes, ponds, harbour basins and sea from ALKIS become uncrossable water (301.1) with a bank line (301.6), with no lines between cadastre areas or along the map edge. Water with a mean width under 2 m comes as candidates. OSM adds ponds, swimming pools, fountain basins, streams, ditches, fountains, wells, springs, drinking fountains on schoolyard maps and bridges on sprint maps as candidates. Contours stop at the shore, and fountain points in water (mostly jets in a basin) are left out. In OCAD the fill and the bank line have the numbers 301.001 and 301.002. An existing map gets the water only with `run --force`.

### Changed

- The north lines are in the map part "default part" with the other map objects, tagged `freemapper:source=north-lines`. In OOM they can be broken at small objects, and on the training map walls and fences lie on top of them. Title, scale and attribution stay in the part "peripheral information". An existing map changes only with `run --force`.
- Colours follow the IOF Printing and Colour Definitions Rev. 4 (September 2024, valid from 1 December 2024) instead of the older values in OOM's symbol set: brown is 25/75/100/0 instead of 0/56/100/18, green 80/0/100/0 instead of 76/0/91/0, brown 50% 10/35/50/0. Buildings are filled with 50 % black instead of 60 %, as in ISSprOM 2019-2. This applies to the OOM map, the OCAD file and the training PDF. Colour names and order stay the same.
- Longer messages use plainer wording.
- If a map area reaches beyond every supported state (abroad, out to sea or into a state freemapper does not support yet, such as Bremen or Thuringia), freemapper warns at fetch that the orthophoto and ALKIS can be missing there. This applies to all states.

### Fixed

- Where the water areas of two states did not quite meet at the border in mid-river (Mainz, Rhine), a bank line ran through the river. freemapper now closes gaps under 0.5 m between water areas before it draws the bank lines. This also applies to parcels of one state. Bridges and roads (0.8 m and wider) stay land, and islands keep their bank lines.

## [0.3.0] - 2026-11-08

### Added

- Hamburg (LGV): orthophoto from the WMS (2026 vintage), terrain and surface models from the state's archives, buildings and land use from ALKIS. Maps use UTM zone 32.
- Saxony-Anhalt (LVermGeo): orthophoto, terrain and surface models (DGM1/DOM1), the nDSM, buildings and land use. Maps use UTM zone 32.
- Hesse (HVBG): orthophoto, terrain and surface models from the municipal downloads, buildings and land use from ALKIS.
- Schleswig-Holstein (LVermGeo SH): orthophoto, terrain model from the INSPIRE ATOM feed (with an XYZ fallback), surface model, buildings and land use from ALKIS.
- Mecklenburg-Vorpommern (LAiV M-V): orthophoto from the WCS, terrain and surface models, buildings and land use from ALKIS.
- North Rhine-Westphalia (Geobasis NRW): orthophoto from the WMS, terrain and surface models (DGM1/DOM1) from laser tiles, buildings and land use from ALKIS, OSM from the extracts of the five Regierungsbezirke instead of the statewide one. Maps use UTM zone 32.
- Maps can now use UTM zone 32 or 33. A project takes the zone of its state, and freemapper reports and cleans up when an existing project changes zone.
- The repository contains one example project per supported state, each with the OpenOrienteering Mapper file, the OCAD file, the training PDF and a preview image.
- The release archives include the user manual with its images, at the path the READMEs link to.
- The README explains the user manual, the example projects and the area selection in the browser.
- This changelog, and architecture decision records (ADRs) in `docs/adr/`.

### Changed

- Every state has one row in the README regions table. Details moved to the user manual.
- Before a download, the size estimate counts whole tiles for states with tiled data and marks the ones already cached.
- Older projects drop their stored records when the raster provider changes, and save that change.
- freemapper pins its Rust toolchain in `rust-toolchain.toml`, so local builds and CI use the same version.

### Fixed

- Hamburg ALKIS falls back to the classic WFS when the OGC API fails.
- State boundaries are valid geometries now, so the check whether an area lies inside a state no longer trips over slivers or self-intersections.
- A server error report is recognised as such and no longer retried, including XML errors from WMS and WCS services.
- Downloads have a size limit, follow no redirects from the Schleswig-Holstein download portal, and leave no half-finished files after a cancelled run.
- Damaged or unusual LZW-compressed TIFF files decode the way libtiff decodes them.
- A failed surface model download keeps a stored model that covers the area. An unreadable flight date no longer aborts a run.
- Missing tiles and places that cannot be resolved show up as warnings with the product and the affected tile instead of ending the run.
- An orthophoto without a flight date is reused for 30 days only, then fetched again. If the flight-date query fails, freemapper keeps a matching stored orthophoto and its date.

## [0.2.0] - 2026-11-07

### Added

- Berlin (Geoportal Berlin): ALKIS buildings and land use in projects, with the matching attribution. Where no ready nDSM exists, freemapper computes it from the bDOM minus the DGM1.
- Saxony (GeoSN): orthophoto, DGM1/DOM1 tiles, the nDSM, buildings and land use from ALKIS, and a Saxony OSM extract. Each region names its own data provider in the map credits.
- Projects pick the data region from the area's centre. Brandenburg (with Berlin cut out), Berlin and Saxony are supported.
- The area selection page and the download estimate know the regions, so they show the right layers and sizes per region.
- A bilingual user manual (German and English) with pictures. It covers installation, the AI assistant, area selection, templates, field checks, the PDF, OCAD and the regions, and it explains the messages. Both READMEs link to it.
- The README says up front that freemapper writes an OCAD starter project and a training PDF.

### Changed

- The README lists regions in one equal table instead of separate Berlin and Saxony sections.
- OSM progress messages name the extract they download (Brandenburg or Saxony).

### Fixed

- Empty answers from the Saxony WFS, tiles at the border outside the raster region and refused requests are handled. A refused request is not retried.
- The Berlin credit line names Geoportal Berlin, and the ALKIS line appears only for Berlin objects.

## [0.1.0] - 2026-11-07

### Added

- First release. freemapper builds an OpenOrienteering Mapper starter project for orienteering from official geodata of Brandenburg and OpenStreetMap, as a command line tool (`init`, `fetch`, `derive`, `export`, `run`) and as an MCP server for AI assistants.
- Georeferenced map with the IOF symbol set for 1:1000 sprint maps, peripheral information (north lines, scale bar, info block) and a credit line that names freemapper and its repository.
- Orthophoto, hillshade and slope as templates behind the map, and contour lines and the nDSM derived from the terrain data.
- Buildings from ALKIS as map objects.
- OSM data as candidates: reliable OSM ways become map objects, the rest stays as candidates for you to review. Geofabrik extracts are cached, and freemapper falls back to the cached extract when Geofabrik is unreachable.
- Sprint map objects: paved areas from ALKIS traffic areas and unpaved paths from OSM.
- Out-of-bounds areas for private land, derived from ALKIS land use.
- OCAD export (`--ocad`, OCAD 12) with placeholder symbols.
- Training map as a one-page PDF (`export --pdf`, `run --pdf`, MCP option `pdf`).
- Area selection in the browser (`freemapper select`, MCP tool `open_area_selector`) with place search, rectangle, polygon, tile outlines and a download estimate.
- Place search through Nominatim (`init --place`) and initialisation from OSM (`init --osm`). The MCP server also lists layers and delivers PNG previews.
- Release archives for Linux (x86_64 and ARM64), macOS (Apple Silicon and Intel) and Windows, each with a SHA-256 checksum.

### Fixed

- Downloads have connect, response and body timeouts, keep a cached copy when the new download is invalid, and clean up old partial files.
- An existing map is kept when you run again. freemapper refreshes its templates and says so.
- freemapper protects edited maps from being overwritten on export.
- Error messages name the product and what to do next. Scale 0 and out-of-range coordinates are rejected.
- Download errors for uncovered tiles come in one report, and offline runs fall back to cached data.
