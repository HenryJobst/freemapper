# Changes

[Deutsche Version](CHANGELOG.md) · [User manual](docs/manual/en.md)

All notable changes to freemapper are listed here, newest first. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.15.0] - 2026-11-12

### Added

- New template local relief (`templates/relief.png`, layer `relief`): the terrain model minus its smoothed mean. Banks, mounds, hollows and ditches stand out light or dark; the general fall of the ground disappears. It is hidden in the map and can be switched on in OOM.
- New candidates from the terrain model (ADR 0104): small knolls (109), small elongated knolls (110), small depressions (111), earth banks (104) and, from 3 m high, impassable cliffs (201). They show what the contours of sprint and schoolyard maps miss: play mounds, sledging hills, hollows, terrace edges. A knoll has to be at least 1 m high on a sprint map and 0.5 m on a schoolyard map, and narrow enough that the contours do not show it already. The ticks of banks point down. None are made at buildings, in water or next to walls, fences and banks already drawn. All of them are candidates to be checked in the terrain (all of them belong to the new group `landforms`, banks and cliffs too; `full` or `--content-set landforms=map` puts them on the map, while `slopes=map` still takes only the banks of ALKIS and OSM); the DGM1 often misses walls narrower than a metre. Knolls and depressions do not come on residential, mixed, commercial and traffic land or in allotments, depressions not within 8 m of a building; cliffs along streets and buildings are offered as impassable walls (515), and bent steps are split into pieces that follow them. `derive` writes them to `data/landforms.geojson`. The ideas come from karttapullautin and omapmaker; the code is written anew.
- Vegetation from laser point clouds (ADR 0105), in Brandenburg and North Rhine-Westphalia: open land (401), runnable forest (405), the green shades 406, 408 and 410, and undergrowth (407, 409) as candidates. The new content group `vegetation` is off in every profile; `--content-set vegetation=candidates` (or `=map`, or the area page) makes `fetch` load the classified point clouds of LGB or Geobasis NRW for the map area into the shared cache, about 160 MB per square kilometre tile in Brandenburg and mostly 50 to 165 MB in North Rhine-Westphalia (each NRW file is checked against the size its index gives), which the download estimate and the disk check count. They are the first part left out when the disk is short, and a failing download keeps the cached tiles. `derive` writes `data/vegetation.geojson`, leaving out synthetic fill points, cellar points and bridges, the returns at buildings and the cells whose high returns are mostly single echoes (roofs, also those the cadastre no longer has); the export cuts the areas at buildings, water and paved areas and, where the map already has a park or wood of the same symbol, shows only the part that differs. Checked on Sanssouci and the Volksgarten in Cologne.
- Vegetation from laser point clouds in Rhineland-Palatinate too (ADR 0105): `fetch` loads the LAZ tiles of LVermGeo RP, mostly 110 to 500 MB per square kilometre, and keeps a tile only when its size and SHA-256 match the Metalink file of the state. The day of the scan comes from the metadata of the tile; when the tile is cached, freemapper asks only for that. Checked on the Stadtpark in Mainz.

### Changed

- Internal: the tests run faster. The dependencies are fully optimized in the development build too (built once, then cached); our own crates stay unoptimized. In CI the `check` job drops from 8 to 10 minutes to 5.5 to 7, the golden net from 2 to 3 minutes to 1 to 2 per system; the outputs of the golden net stayed byte-equal. CI writes line numbers only as debug information, and a newer push to a pull request cancels its older run. A pull request runs on Linux only; with the label `all-systems`, and on every push to main, all three systems run. The first debug build after updating compiles the dependencies once more; the release build does not change.
- All twelve examples are rebuilt with the default profile `balanced`. Ten of them predated the content profiles; now contours, paths, stairs, walls and trees are on their maps, and the candidates are mostly out-of-bounds areas, control sites and earth banks. The [comparison page](examples/compare.en.md) shows one area, Cochem, in the three profiles `safe`, `balanced` and `full`, with a PDF each. The numbers in the examples overview and the manual are counted again.
- MCP: what the server finds wrong with the person's adapter folders at its start (a folder left out, or one that loads with a caveat) now comes once as a note with the first tool result. Before, it went to stderr only, which neither the agent nor the person sees.
- MCP: `preview_layer` with the layer `map` shows the image of the map objects that the last run wrote next to the map (without the candidates). Before the first run the message names `freemapper export` or `run`.
- Two adapters share ground when their common area is more than a hundredth of the smaller region. Before, one inner point of each was tested, so an adapter that reached a third of the way into a state went unnoticed. `adapters list`, `adapters check` and the warning about the adapters of a project use the new measure. All three now name the regions by name, not by id; the JSON of `adapters list` keeps the ids.
- The message about an adapter the program does not have says that a newer freemapper may bring it.
- The message about a redirect freemapper does not follow starts with the refused address and then names, in brackets, the address that redirects there. Before, the address that was asked came first and was called "this address".
- The example [Treppenviertel Blankenese](examples/hamburg-blankenese/) is rebuilt with the default profile `balanced`: contours, paths and 110 stairs from OpenStreetMap are on the map now (1780 map objects, 755 candidates). The [comparison page](examples/compare.en.md) says which examples were made before the content profiles and gives the measurements for Blankenese and Saarlouis.
- Internal: after the adapter work the golden net still records four states (Brandenburg, Berlin, Saxony-Anhalt, Hesse), one of each kind of fetch; the layers that need no recording still check all eleven states that were program code before the rework (Saarland has tests of its own). The fixture store of the tests shrinks from 206 to 79 MB.

### Fixed

- Map preview: when the area page is open in two tabs and both change a setting shortly after each other, the overtaken tab no longer shows "A newer preview was asked for meanwhile". It asks again and gets the image of its own settings.
- MCP: `list_layers` lists the K5 template only for projects whose area touches Berlin, or when its file is in the folder. Before, every project listed it, and outside Berlin it never got ready.
- Saarland: a district archive that cannot be read (renamed, or its server down for a moment) no longer stops the fetch when the tiles of the area are in other archives. When an archive is down, its directory from the last run is used; maps whose tiles are in the cache already are still made. A warning with the address of the archive says so in both cases. A cancel, an address freemapper does not ask and a write error on this computer still stop the fetch at once.
- A folder in the adapter folder whose name starts with a dot (`.git`) no longer causes a warning on every command.
- A pre-release version of the program (`x.y.z-rc.1`) no longer aborts while loading the adapters; it counts as its release.
- An estimate with nothing left to download says 0 MB, not `-0.0` (`estimate_download` of the MCP server).
- In the release archive the links of the READMEs to the examples, the ADRs and the design led nowhere. They now point to the repository.

## [0.14.0] - 2026-11-12

### Added

- Saarland is the twelfth state (ADR 0111). The orthophoto of 2025 comes from the free map service of the LVGL, the terrain and the surface model of the 2025 laser scan from the district archives of its open-data share (freemapper reads only the 1 km tiles of the area out of archives of up to 1.3 GB), buildings and land use from the service "ALKIS Vereinfacht" of the Geoportal Saarland. All of it is dl-de/by-2-0; maps carry `© GeoBasis DE/LVGL-SL (<year>), dl-de/by-2-0, Daten geändert`. The example is [Saarlouis old town and fortress](examples/saarland-saarlouis/). Saarland is the first state added as an adapter: a manifest folder and one raster backend.
- Region adapters from a folder (ADR 0082, stage D; ADR 0107). A state or country with standard services (WFS or OGC API Features for ALKIS, WMS and WCS for the rasters) can be added without Rust: a folder with a `manifest.toml` and a boundary file. freemapper reads such folders from your adapter folder (`freemapper/adapters` in the configuration folder of the system, or the folder `FREEMAPPER_ADAPTER_DIR` names). A folder with a defect is left out with a warning that names the file, the line and the key. An adapter from a folder cannot replace a built-in one. The reference in `docs/adapters/` describes the format; the manifests in `crates/adapters/de/manifests/` are examples.
- `freemapper adapters list` shows every adapter with where it comes from, its revision, the day it was last checked and the hosts it asks; `--json` prints the same as JSON. An adapter that lies on the ground of another one names it, because a map there gets the objects of both.
- `--adapters-from-project` for `init`, `fetch`, `derive`, `export` and `run` also loads the adapters in the folder `adapters` of the project folder. A `warning:` line names each with the hosts it may ask. The area page and the MCP server never load them, so a project someone passed on cannot bring a data source along.
- `freemapper adapters check <folder>` checks the folder of an adapter before it is used or passed on (ADR 0112): whether it would load, and what its author should know (no day of the last check, an id that is built in, a boundary of an odd size). With `--live --bbox minLon,minLat,maxLon,maxLat` it also asks the services for that box and prints one line per capability, for example `rasters ok: orthophoto, terrain model, surface model`. Exit code 0 means it passes, 1 it has findings, 2 the check could not run; `--json` prints the result as JSON.
- `freemapper adapters allow-host <adapter> <host>` allows a built-in state one more host, for the day an agency moves a service to a server the state's adapter does not name yet (ADR 0110). The message about the refused address names the host and the exact command. The host is kept in `allowed-hosts.toml` in your configuration folder and nowhere else: a project folder, a variable or the MCP server cannot add one. It is checked like a host of a manifest (a DNS name, no address, no port), and freemapper still connects to servers on the internet only. `adapters list` shows the allowed hosts, `adapters disallow-host` takes one back, and once a release names the host itself freemapper says so and removes the entry.
- A guide for contributors in `docs/adapters/` (German and English): how to add a state or region, a tutorial that builds an adapter in six steps and ends with a check that passes against real services, and a reference of every key and finding code of the manifest. The guide also says when a manifest is not enough: height models that come as archives or tiles need a backend in Rust, and a folder cannot bring one.
- `FREEMAPPER_ADAPTER_DIR` takes an absolute path. A relative one is ignored with a warning: it would name the folder `adapters` of whatever project the command runs in.
- OSM stairs (`highway=steps`) appear as stairway candidates 532.6 to 532.9 by width, turned round for `incline=down`, without closed, covered, underground and indoor stairs (ADR 0109). With `balanced` they go onto the map (group `stairs`). An official stair with the same symbol wins. Before, stairs came only from the Berlin registers and the full ALKIS; Hamburg had none.
- A preview of the map (ADR 0108). Every export that writes a map also writes `<name>-preview.webp` next to the `.omap`: the map objects as the map PDF draws them, no candidates, no sheet. The area page shows it in a new section "Map preview" below the run status, and when you change the content profile or a group there, it shows the effect without a run and without saving: within about a second, after a first computation that takes a few seconds. A line says whether the image is the map as generated last or the settings on the page; after a change of area, map kind or scale, or while a drawn area is not saved, it says that a run with "Replace the existing map" is needed. A later fetch does not make the image out of date. If the preview cannot be written, the run says so in a warning and still writes the map, the OCAD files and the PDFs; the list of files of a run names the preview. The `.omap`, the candidates map, the OCAD files and the PDFs do not change.

### Changed

- Requests go to servers on the internet only (ADR 0106). freemapper no longer connects to an address of this computer or of the local network, whether a URL names it, a host name resolves to it or a server redirects to it. This holds for the downloads, the map tiles of the area page and the self-update. A proxy from the environment (`HTTPS_PROXY`) still works, also a local one. A request that is not sent says why, in the language of the run.
- The requests for a state stay on the hosts its manifest declares, redirects included (at most three): ALKIS, the rasters, the full ALKIS, the tree cadastres, and Berlin's registers and K5 map. A server that sends freemapper on to a host nobody declared gets no request there; the same holds for a fourth redirect in a row, a redirect to `http` and an address in an answer on such a host. The message starts with the refused address, names the state and says what to do. An adapter from a folder is held to its own hosts in the same way and adds none to a built-in state. Before the switch every state was fetched live at two places under this rule.
- Internal: the eleven states are manifests plus code (ADR 0107). Each has a folder in `crates/adapters/de/manifests/` with its manifest and its boundary file; Saxony-Anhalt needs no raster code any more. The golden net confirms the same requests and byte-identical files.
- When a project file names an adapter this freemapper does not have, the message says where adapters come from.
- Internal: the states are region adapters now (ADR 0082, stages B and C; ADR 0102). A state supplies its boundary, its OSM extracts, its ALKIS service, its raster sources, its credit lines and its download estimate through one trait; the engine asks the registry and names no state itself. The enums `Region` and `Provider` are gone. The code of the eleven states lives in the new crate `freemapper-adapters-de`, the generic service readers in `freemapper-sources`, the shared types in `freemapper-adapter-api`. Nothing changes for users: the golden net confirms the same requests and byte-identical files for all eleven states, and `freemapper.toml` is read and written as before.
- The area page, its content security policy, the message for an area outside the supported states and the MCP instructions take the list of the states from the registry (ADR 0103). The aerial photo layers of the page come from the new endpoint `api/adapters`; their names are the same. When the endpoint fails, the page still starts, shows OpenStreetMap only and says why.

### Fixed

- When a project file names a provider or a state this freemapper does not know, the error says so and names the id. Before, it was a parser message about an unknown value.
- Tests: `save_creates_the_project` failed now and then on Windows and Linux. The server answers a POST to a GET route with 405 before it reads the body, and closes the connection when the body arrives only afterwards. The test client kept using that connection. The check now sends the request raw, in one piece. The program itself does not change.
- Tests: on Linux the update tests failed now and then with "Text file busy". The test process wrote the copy of the program they start itself, and a child process another test started at the same moment inherited the open file. Now `cp` writes the copy in a process of its own. The program itself does not change.
- The area page shows the disk space notes in its own language: what is left out for lack of space, why a run does not start, and that the free space cannot be read. Before, they came in the program's language, so a German page often showed them in English. The same goes for the refusal of an invalid `pdf_sources`.

## [0.13.0] - 2026-11-12

### Added

- The area page offers the content profile (ADR 0101): a collapsed section "What goes onto the map" in the "Create the map" block with the three profiles and one choice per group ("As in the profile", Map, Candidate, Off). The choice is saved with the area, in one write under the revision of `freemapper.toml`; a change of the profile keeps the exceptions.
- `freemapper find "<name>"` lists the OpenStreetMap places for a name, up to five, those with an area first, each with its id, and ends with the `init --osm` command for the first area. The line of a match without an area says that it cannot be a map area. `--json` prints the same fields as the MCP tool `find_place`. When nothing is found, the command exits with 1; with `--json` it prints `[]` and exits with 0, as `find_place` answers. Control characters and bidi overrides from OSM names do not reach the terminal. It runs the same search as `find_place` and `init --place`, and `init --place` stays as it is. Like `init`, `find` runs the daily update check. `AGENTS.md` now tells agents on the command line to show the person the matches first and then use `init --osm`.

### Changed

- Content profiles (ADR 0100): what goes straight onto the map and what stays a candidate is adjustable: `safe` (the former state), `balanced` (the new default: streets with kerbs and green strips and stairs from ALKIS, the registers and the street survey, and walls, fences, hedges, water, trees, tree groups, paths, railways, banks, passages, monuments, towers and above-ground pipelines also from OSM, and the contours and the areas that shall not be entered along railway tracks, go onto the map) and `full` (everything). Single groups can differ. `init`, `export` and `run` take `--content` and `--content-set <group>=map|candidates|off`, MCP `content` and `content_set`; the choice is saved in `freemapper.toml` (`[content]`). Contours now also stop at ways when they are on the map.
- Paved areas (501.5) get their black border 501.1, as the norm (ISSprOM §4.5) has it (ADR 0099). No border is drawn where an edge already shows: along building outlines, water, existing 501.1 lines (the kerbs of the street survey) and the edge of the map. Paved OSM ways of the class `service` (driveways, courtyard roads) come in as 501.6 to 501.9, 501.8 without a width.
- OSM ways with a paved surface (asphalt, paving stones, concrete, sett …) now come into the map as lines 501.6 to 501.9 with the black border, like gravel paths (ADR 0098). Before, they were missing where the ALKIS and Berlin areas do not hold them, such as park paths in the Biesdorfer Park; where a paved area lies, `build/firm_paths.rs` cuts them. Streets and sidewalks keep no rule.
- All core crates are pinned to freemapper-core v0.21.1. The buttons of the area page take their text colour from the suite token `--on-accent`, and the main buttons their colours from the suite class `button.primary`. They look the same as before.
- Point symbols (trees, lamps and others) keep the norm's gap to walls and fences: one that comes closer than 0.15 mm between the outer edges is moved away from the line to that distance, and also keeps the gap to other point symbols (the nearest free spot along the fence or further away). A moved point that runs into the same symbol is left out instead, and the other names it in `freemapper:generalized`. On the tick side of a fence the ticks count. Moved points carry `freemapper:moved`.
- Contours and north lines now stop at ways (paved areas and paths) instead of covering them; leftover pieces under 1 mm on paper go. The norm allows it: north lines may be broken where they obscure small features such as path ends (ISSprOM 2019-2 §4.6). ADR 0097.
- The ticks of fences and banks now point the right way (`build/ticks.rs`). A fence turns them to the olive green area (520) beside it; a closed fence, a rectangle say, has them inside, and a fence along a park area (401) on the park side. The ticks of earth banks (104, 104.1), impassable cliffs (201) and retaining walls (513.2) point down the slope, by the terrain model; when it is missing or the ground is flat, a bank takes the direction of the already decided bank beside it, so neighbouring banks no longer face each other. Before, the lines kept the direction of their source.
- The documents of a project and the starter project, OCAD and PDF exports moved from `project.rs` into their own module `project/export.rs`. The public paths and the output stay the same; the golden net (G3, G4 for all states, G5) confirms it byte for byte.
- The area page uses the suite's shared page frame from freemapper-core v0.21.0 (ADR 0096, core ADR 0030), which freemapper-courseplan and freemapper-print take up too. It now has a dark mode that follows the system setting; the map itself stays light. Two open windows of the same project, or the page and a run in a terminal, can no longer overwrite each other's `freemapper.toml` unnoticed: when the file changed since the page read it, the page saves nothing, says so in a banner and reads the saved state again; only a second "Save" replaces it, and until then "Create map" does not save on its own. The page's texts now live in the program's catalog (`page-*`), word for word as before; the server picks the language as before (`--lang`, then the environment, then the browser, then English), from the browser now the German or English entry of `Accept-Language` with the highest weight instead of only the browser's first language. The page writes the numbers in its texts for its language, so a German page writes "1.234". All core crates are pinned to v0.21.0. A script that calls `api/save` for an existing project sends the `revision` from `api/project` in the header `X-Freemapper-Revision`.
- The area selection page now tells when only the data provider changes with a new area (same UTM zone, other state): it showed the plain "saved" message although freemapper had dropped the fetched data. `set_map_area` reports the reason (`zone_changed`, `provider_changed`), and the answer of `api/save` carries `provider_changed`.
- The download estimate as JSON (`estimate_json`) moved from the selection page module into its own module of the command line crate. The MCP server and the browser run no longer depend on the web module for it. No change in output.
- The project file and the map no longer depend on the maths library of the system: `freemapper.toml` stores the areas of the orthophoto, the terrain model and the surface model rounded to the millimetre and the area read from OSM to 0.00000001°, and the map's reference point is written to the `.omap` rounded to the millimetre. Before, these numbers differed in their last digit between Linux, macOS and Windows. Existing projects stay valid; a stored area still covers the map area when it is half a millimetre short.
- The library asks for the date and for a file's age in one place (`freemapper::clock`). For users one edge case in Rhineland-Palatinate changes: a kept tile list or 404 note whose file date lies in the future now counts as fresh, not as outdated.
- Internal: a golden net guards the change from the region enums to region adapters (ADR 0082, stage A). Five layers record what v0.12.0 outputs: the facts per state, how 211 areas on borders are assigned, the requests of a fetch per state, the checksums of every file of a run per state, and the format of the project file. The recorded answers of the services live outside the repository as the prerelease `golden-adapters-1` (206 MB); `scripts/golden.sh test` fetches them and runs all layers. A CI guard requires the trailer `Golden-Bless:` for every change to the golden files and refuses one in a `refactor(` commit. Details in `tests/golden/adapters/README.md`.

### Fixed

- When the newest release has no update manifest `freemapper-update.json`, `freemapper update` and the MCP tools `check_update` and `install_update` now say that it cannot install itself and link its release page. Before, they printed a raw "download failed: … HTTP 404". With `--version`, the message says that the release may not exist. The daily check stays silent in this case, as before.
- The daily check after `init`, `select`, `fetch` and `run` now reports a release whose signature does not match. It prints one warning line on stderr after the command's output, at most once a day, and the exit code stays the same. Before, it said nothing. The MCP server gives the same news once, as a note for the person.
- The server of the area page now refuses in the page's language, like the page's texts. That covers a save during a run, a folder held by another run, an area or map kind the server does not take, and the answers to run, status, cancel and open folder. Before, they came in the program's language, and the refusal during a run was always English.

## [0.12.0] - 2026-11-11

### Added

- Self-update (ADR 0094): `freemapper update` asks, then installs the newest release over the running program at the same path, so the registration with the AI assistant stays valid. `--check` only checks and exits with 10 when a newer version exists, otherwise with 0. `--yes` installs without asking, `--version <x.y.z>` picks a release, also an older one, and `--rollback` goes back to the replaced version, which stays as `freemapper.previous`. freemapper installs only what is signed with the project's minisign key and matches its checksums, and it starts the new program once as a test before the swap. After `init`, `select`, `fetch` and `run` it checks at most once a day and reports a newer version in one line on stderr after the command's output; a very fast command can take up to 3 s longer for it. The update mode (`notify`, `install` or `off`) lives in `settings.toml` in the system's configuration folder, is set with `freemapper update --mode` and can be overridden with `FREEMAPPER_UPDATE`. `--offline` or a set `CI` means no check. The request goes only to github.com, with the fixed `User-Agent` and no personal data. The MCP server has the tools `check_update` and `install_update` (only with `confirm: true`) and adds a note for the person to its next tool result when it knows a newer version; it never installs on its own. The area page then shows a line with the command. A build from source names `git pull` and `cargo build --release --locked` instead. 0.12.0 is the first release that can update itself; 0.11.0 and older are replaced by hand once. The manual (chapter 3, "Updating"), the READMEs and `AGENTS.md` describe it.
- Each release also carries the bare executables as `freemapper-<tag>-<platform>.gz` and the update manifest `freemapper-update.json` with its minisign signature `freemapper-update.json.minisig`, which `freemapper update` loads. The archives stay as they were.

### Changed

- `AGENTS.md`: agents that can run commands use the command line first. One `freemapper run` waits until the map is written, while through MCP every status poll costs a turn. MCP stays the way for assistants without a shell. New: a command-line workflow, the pointer to `--help`, the note on too short timeouts, and that `init --place` takes the best match without naming the others.
- Map kind and reserved words come from the shared crate `freemapper-contract` (ADR 0095, stage 4 of the contract test). The names `schoolyard` and `sprint`, the scales of a kind, the tag `freemapper:source` with its reserved values, the part names and the file name `freemapper.toml` now exist once, in freemapper-core v0.18.0, and freemapper-print and freemapper-courseplan can use the same definitions. Maps and project files stay the same byte for byte. All core crates are pinned to v0.18.0. Library: `MapKind` gains `ALL`, `name`, `from_name` and `contract`.

### Fixed

- Area page: when the program language differs from the browser's language (for example `--lang en` in a German browser), three texts stayed in the browser's language: the hint under "Draw it yourself", the names in the map's layer control and the title of the browser tab. All three now follow the program language.

## [0.11.0] - 2026-11-11

### Added

- Program language, third stage (ADR 0092): progress lines and warnings come in the program's language, on the command line, in the MCP server and in the log of the area page. A German warning starts with `Warnung:`. An error inside a warning, such as a failed download, is translated as well. That includes the warnings about the height models of the single states (tile lists, dates).
- Program language, help (ADR 0092, stage 4): `--help`, `freemapper help <command>` and the messages about wrong calls (unknown option, missing argument, invalid value) come in the program's language, with German headings ("Aufruf:", "Optionen:"). The English help is unchanged.
- Program language, MCP server (ADR 0092, stage 4): the server instructions, the descriptions of the tools and their parameters, the results in words and the summary at the end of a run come in the program's language; the area page shows the same summary. Names of tools, parameters and JSON fields stay English. The note after `init` about 1:5000 on sprint maps is translated too. `freemapper mcp --lang en` keeps the server English.
- Program language, remains (ADR 0092): the messages of the place search (such as "… is an area of 3 separate parts"), the messages about layers and previews and the warnings of the OCAD export now have German texts. The message table in the German manual gives the German wording with the English one in brackets.
- `AGENTS.md`: instructions for AI agents on installing freemapper (choosing the archive, the checksum, the macOS quarantine flag) and using it through MCP or the command line, with the tools, the workflow, the result files and what the agent should tell the person. It is now part of every release archive too. The manual (chapters 3 and 4) and the READMEs tell how to let an assistant that can run commands set freemapper up with it.

### Changed

- Library: the progress callback receives a `Note` (a message id and its values) instead of a finished line (`&str`); `to_string()` gives the English line as before, `Note::text` the line in a language, and `Note::is_warning` tells whether it is a warning. `HeightModel::warnings` and `OcadOutcome::warnings` hold such values instead of strings, and so do `AreaError::Place` and `ProjectError::Layer`; `MapKind::scale_note` returns one. The texts live in `crates/freemapper/i18n/`.

## [0.10.0] - 2026-11-11

### Added

- Area selection: the page now creates the whole map (ADR 0090, stage 5; ADR 0009, amendment). Below the settings sits the block "Create the map" with the options OCAD starter project, printable PDF, training PDF, scale bar, source lines (automatic, full, freemapper only, with a hint about the ODbL and dl-de/by-2-0 licences), "Cache only" and, only when a map exists, "Replace the existing map" (never ticked in advance, with a confirmation dialog). The page saves first when needed, shows "Step i of n", the log excerpt and the warnings, can be cancelled, lists the files with their size at the end and opens the project folder in the file manager. Reloading during a run picks the display up again. While a run goes on, the area, the settings and "Save" are locked. The defaults come from `freemapper select --ocad --pdf --pdf-candidates --pdf-scale-bar --pdf-sources --offline`. All texts exist in German and English by the browser language; a `lang` field from the server would win, but it is still empty. The scales of a map kind now come from the server (`allowed_scales`) and no longer from a copy in the page. New manual section: "Create the map in the browser".
- Area selection: the local server has routes for the run (ADR 0090, stage 4). Scripts can use them now; the page follows in stage 5. `POST api/run` starts the run with the seven options (`force`, `offline`, `ocad`, `pdf`, `pdf_candidates`, `pdf_scale_bar`, `pdf_sources`) and answers 202, or 409 when a run or another process already holds the folder; `GET api/status?since=N` reports the step, new lines, warnings and files, `POST api/cancel` stops the run, `POST api/open-folder` opens the project folder in the file manager. Kind, scale and title stay as saved. `freemapper select` and the MCP tool `open_area_selector` take `--offline --ocad --pdf --pdf-candidates --pdf-scale-bar --pdf-sources` as the page's defaults.
- One project folder, one run (ADR 0090, stage 3; closes the gap named in ADR 0009). `run`, `fetch`, `derive`, `export`, the MCP pipeline and saving the area on the page create the file `.freemapper-run.lock` in the folder. A second process in the same folder stops with a clear message, for example "another freemapper run is using this folder (since 14:02, started by the terminal)". The holder reports in every ten seconds. After 60 seconds without a report the lock counts as abandoned, after a crash say, and is replaced; Ctrl+C clears it at once. The file holds only the process id, the start time and who started it. The engine can also stop between steps, tiles and requests now (`ProjectError::Cancelled`, `run_project_cancellable`). A cancelled run leaves only whole files, and the next run continues from there. The cancel button on the page follows in stage 4.
- Railway tracks from OSM (ADR 0093). On sprint maps, `railway=rail`, `light_rail` and `narrow_gauge` are now map objects 509.1 (railway, at least 4 mm long), bridges and sidings included. Before, freemapper read no track at all: the map of the Biesdorfer Park had no S-Bahn. Tunnels, passages under buildings, disused, razed and planned tracks and crossovers are left out. Subway tracks that are not in a tunnel are 509.1 candidates, on an elevated viaduct map objects. Tram tracks are 509.2 candidates, because the norm draws them only when they help navigation. A forest is cut at a track. The railway land of ALKIS stays a 520 candidate. Schoolyard maps have no tracks. Three examples change: Viktoriapark (3 tracks), Herrenhausen (2) and Cochem (4); Herrenhausen and Cochem also get the firm gravel paths of the previous fix. The other eight have no track in their area; Quedlinburg could not be rebuilt (the Saxony-Anhalt orthophoto service answered with an error). Existing maps get the tracks with `run --force`.
- Program language, first stage (ADR 0092): `--lang de` or `--lang en` chooses the language of the program texts; without it `FREEMAPPER_LANG` counts, then the system language (`LC_ALL`, `LC_MESSAGES`, `LANG`), then English. German so far covers the lines the program prints itself at the end of a step ("… geschrieben", "… behalten", the hints after `run`, `export` and `select`). Progress lines, warnings, error messages and `--help` stay English for now.
- Program language, second stage (ADR 0092): error messages come in the program's language, in the terminal, from the MCP server and on the area page. With `--lang de` an error reads, for example, "Fehler: Es sind noch keine Kacheln geladen; rufen Sie zuerst `freemapper fetch` auf". What a server, the operating system or a parser says about an error stays as it is, and so do a few details freemapper still keeps as free text (for example of the place search). Progress lines, warnings and `--help` follow in the next stages.
- Area selection: the page takes the program's language (ADR 0092) when `--lang`, `FREEMAPPER_LANG` or the system (`LC_ALL`, `LC_MESSAGES`, `LANG`) names one. `freemapper select --lang en` shows the page in English in a German browser too. When nothing names a language, the page follows the browser as before.

### Changed

- An error in the terminal is now one line, `error: …` or `Fehler: …`, with the cause after a colon. Before, it read `Error: …` with the cause below under "Caused by:". The exit code stays 1.
- `export` and `run` build the map and the candidates template once per command instead of once per output (ADR 0091). With `--ocad --pdf --pdf-candidates` that used to be four passes over all stored layers; now it is one for `export` and two for `run`. The written files are the same byte for byte.
- Area selection: the local server now runs on the shared library `freemapper-web` from freemapper-core v0.11.0 (ADR 0090, stage 1; ADR 0009 and 0067, updates). The page looks the same. The checks are stricter: a POST needs the page's `Origin` and `Content-Type: application/json`, or the server answers 403. A script that calls the routes must send both. Requests with `Sec-Fetch-Site: cross-site` are refused, every response carries `X-Content-Type-Options: nosniff`, and a request body may be at most 1 MiB (an outline with 5000 points takes about 150 KB). All core crates are now pinned to the same tag v0.11.0.
- The example preview images are now WebP instead of JPEG (`preview.webp`, `preview-map.webp`). They are rendered straight from the PDFs and are smaller than the JPEGs. The README title image (Viktoriapark) is sharper at 1600 pixels wide.
- Tests: every test run now checks that the map and `freemapper.toml` keep the suite contract (ADR 0089, `freemapper-contract` from freemapper-core v0.8.0). A schoolyard map at 1:1000 and a sprint map at 1:4000 are tested against twelve rules and against the vocabulary freemapper-print and freemapper-courseplan rely on: part names, the tags of the peripheral information and of the walkable area, the contract keys of the project file. The program itself does not change.

### Fixed

- Firm gravel paths (501.6 to 501.9) stay whole where only an ALKIS `Weg` parcel lies (ADR 0079, addendum). That parcel is only a 501.5 candidate, not a map object, yet it cut the OSM line away. Now only 501.5 map objects (roads, squares, the survey) cut. The 0.25 mm tolerance and the 1 mm minimum piece stay. Existing maps get this with `run --force`.

## [0.9.2] - 2026-11-10

### Fixed

- Area selection: in some browsers the base map showed "access blocked" (403) from OpenStreetMap (issue 11). The page sends no `Referer`, and OpenStreetMap's tile policy asks for a valid `Referer` or an identifiable `User-Agent`. Now the local server fetches the tiles itself (`/tiles/osm/{z}/{x}/{y}.png`) with `User-Agent: freemapper/<version> (+https://github.com/HenryJobst/freemapper)` and nothing else. It checks zoom (0 to 19), column and row, follows no redirect, makes at most two requests at once and keeps the tiles in memory for as long as OpenStreetMap allows (at most 1024 tiles, 32 MB). The page no longer loads anything directly from `tile.openstreetmap.org`. The state aerial images stay as they are (ADR 0088).
- Wide stairs such as stadium stands (ADR 0048, amendment): the parallel 532.9 lines now stand one symbol width plus 0.15 mm apart, the norm's minimum gap between two lines of one colour. Before, the ticks of neighbouring lines met and the stand looked like a ladder. The width stays covered to its edge. The Viktoriapark goes from 94 to 86 lines 532.9. Existing maps get this with `run --force`.
- Berlin: fountains, street pumps and drinking fountains no longer appear twice (ADR 0087). Measured in the Viktoriapark: a surveyed pump and the OSM well at the same spot lie 0.05 to 0.5 m apart, the next point of another source 9.8 m. The limit is 0.75 mm on paper (3 m at 1:4000), never under 3 m on the ground. The order is register, ALKIS, street survey, OSM. The street survey now beats OSM wells (`man_made=water_well`, `amenity=fountain`) and OSM drinking water (`amenity=drinking_water`), and the Zierbrunnen register beats an OSM well as well. The winner names the dropped point in `freemapper:also`. A register, ALKIS or street survey fountain inside an OSM fountain basin (`amenity=fountain` as an area) or within 1 m of it is dropped, and the basin names it. Two fountains under the limit without a higher source both stay. In the Viktoriapark 9 candidates 311 remain (3 register, 4 survey, 2 OSM) and 0 candidates 313; only the note changes from `freemapper:generalized` to `freemapper:also`. Existing maps get this with `run --force`.

## [0.9.1] - 2026-11-10

### Changed

- Coordinate systems and tiles now come from `freemapper-georef` (freemapper-core v0.5.1). `crs` and `tiles` in freemapper re-export the types under the old paths, and nothing changes in behaviour. The projection uses the shared transverse Mercator code. `freemapper-fs` is now on v0.5.1 too.

### Added

- Power lines, pipelines, towers and masts from OSM, sprint maps, candidates only. Overhead `power=line|minor_line` and `power=cable` with `location=overhead` are 510, lines of 110 kV and more 511. Pipelines with `location=overground|surface|overhead` are 528 (low) or 529 (high, uncrossable: overhead or `height` of 1.5 m or more). `power=tower`, `man_made=communications_tower|chimney|water_tower|lighthouse`, towers and masts are 524 or 525 points by height. Underground lines and pipelines, poles, masts under 10 m and storage tanks are left out. The lines keep the norm's minimum length (3.75, 2.2 and 3 mm on paper). Each tower point carries `freemapper:control=` and is thinned like the street furniture. ALKIS: the layers we fetch hold no towers or lines, so there is no ALKIS rule (follow-up in the ADR). No example area has a line or a pipeline. Wismar, Monschau and Quedlinburg get one tower each (candidates 1756 to 1757, 850 to 851, 1110 to 1111) (ADR 0083).

### Fixed

- The scale bar carries `freemapper:source=sheet` again, like every sheet object (since 0.9.0 it said `scale-bar`). freemapper-print and freemapper-courseplan recognise the sheet only by that value; otherwise they printed the bar as map content and shifted the page centre and the extent. Every bar object also carries `freemapper:sheet=scale-bar`, which the PDF uses to leave the bar out without `--pdf-scale-bar`. The OOM and OCAD maps keep the bar. Maps already written with the old value stay readable (ADR 0080, amendment).
- Duplicate areas by rank (ADR 0078): an area that was itself dropped for a higher one no longer drops another. In a chain A > B > C, C used to go because of the already removed B, and the `freemapper:also` note landed on B. Now the area that stays on the map carries it, and C stays when A alone does not reach the share. Existing maps get this with `run --force`.
- The map area is checked on every path, not only on the selection page (ADR 0084). `init --geojson`, MCP `create_project` with `geojson`, `init_project` and the area in `freemapper.toml` now refuse a polygon without corners, with crossing edges or with more than 5000 corners, with "not a usable map area". An empty polygon (`"coordinates": []`) used to crash the server.
- `init`, MCP `create_project` and the area selection page take only scales that freemapper-print prints (ADR 0085): schoolyard 1:1000, sprint 1:4000 or 1:5000. Before, `--kind schoolyard --scale 2000` ran through and freemapper-print then stopped with `MapScaleNotAllowed`. The error lists the allowed scales; for sprint 1:3000 it advises drawing at 1:4000, because freemapper-print enlarges to 1:3000 when printing. 1:5000 is not in the norm; `init` says so, and the symbols stay at 100 %. A project with another scale in `freemapper.toml` still loads and prints a warning. The selection page offers the allowed scales as a list.
- `init` always writes a `title` into `freemapper.toml`, also without `--title`: the folder name with capitalised words, as the PDF already built it (`Clara-Zetkin-Schule`). freemapper-print and freemapper-courseplan now read the same title (ADR 0086). Older projects without a `title` stay as they are.

## [0.9.0] - 2026-11-10

### Changed

- All eleven examples are rebuilt with the current code. The PDF of each example is the training map (`<name>-candidates.pdf`), and `preview.jpg` is rendered from it, because the map alone is very plain. Map objects, candidates and symbols in the examples overview are counted again, and the gravel paths 501.6 to 501.9 are map objects in every example. Only Cochem and Blankenese also have the map without candidates (`<name>.pdf`, `preview-map.jpg`). The Viktoriapark attribution names the public lighting register again.
- The default of `--pdf` changes. It now writes `<name>.pdf` with exactly the objects of the OOM map, without candidates and without a scale bar. The training map with every candidate is `<name>-candidates.pdf`, written with `--pdf-candidates`; both switches together write both files. The scale bar on the PDFs only comes with `--pdf-scale-bar` (German orienteering maps usually have none); the scale number stays in the title block. The old behaviour is `--pdf-candidates --pdf-scale-bar`. This also holds for `run` and the MCP parameters `pdf`, `pdf_candidates`, `pdf_scale_bar`. The OOM and OCAD maps keep the bar. Amends ADR 0002 (ADR 0080).
- The Viktoriapark training map (`berlin-viktoriapark-candidates.pdf`, `preview.jpg`) is regenerated with the shortened source block (four lines instead of nine). The other examples have at most five lines and stay unchanged; the map content is identical.

### Added

- The source block on the PDF map is shortened when needed. `--pdf-sources auto` (the default, also MCP `pdf_sources`) prints up to five lines unchanged; a longer block gets the `dl-de/zero-2-0` sources merged for each provider, long names shortened, lines joined with " · " and, last, a pointer to `ATTRIBUTION.txt`. OpenStreetMap, every other licence and the freemapper line stay word for word. `full` never shortens; `credit-only` prints only the freemapper line and says in the output that ODbL and `dl-de/by-2-0` require naming the sources when you pass the map on. The freemapper line cannot be switched off in any mode. `ATTRIBUTION.txt` and the OOM and OCAD maps always list every source. The Viktoriapark block shrinks from nine lines to four (ADR 0081).
- A comparison of map and training map: [`examples/compare.en.md`](examples/compare.en.md) shows Cochem and Blankenese side by side, with the most frequent candidate symbols and the measurement on all eleven examples that decided the choice. The examples overview and the README link to it.
- Berlin control sites on sprint maps. The street survey (bollards, masts, flag poles, benches) and the lighting cadastre (lamps in operation) now give 531 candidates with `freemapper:control` (`bollard`, `mast`, `flagpole`, `bench`, `street_lamp`, plus `advertising_column` for the survey's advertising columns), like the OSM ones. Thinning is shared: a site of one kind within 25 m of another is left out, whichever source it comes from, after duplicates and sites in buildings are gone. A surveyed bollard replaces an OSM one within 2 m. Hydrants and play equipment stay schoolyard-only. A Berlin sprint project fetches seven more WFS requests (33 instead of 26); the disk estimate for the Berlin layers goes from 10.5 to 12 MB per km². Existing maps get this with `run --force` (ADR 0077).
- Drinking fountains are 313 on sprint maps, from the Berlin register and survey and from OSM `amenity=drinking_water`.

### Fixed

- Firm gravel paths are map objects again. `surface=compacted|fine_gravel` and `tracktype=grade2` (paved by ISSprOM 501, "hard gravel") are drawn as a line 501.6 to 501.9 by `width` on the map, no longer only as a candidate; the park maps have their paths back. Where ALKIS or the street survey already draw paving (501.5), the part of the line within it goes (widened by 0.25 mm on paper), and remnants under 1 mm on paper go. Ways without `surface`, unknown surfaces and `tracktype=grade3` to `grade5` stay candidates, and tree groups are still cut at the lines. Viktoriapark: 26 map objects 501.6 instead of 49 candidates, Sanssouci: 192 instead of 218. Amends ADR 0073 (ADR 0079).
- Duplicate areas from several sources are kept once. An OSM passage (`tunnel=building_passage`, `covered`) over a Berlin building part, an ALKIS passage or a canopy, an OSM private strip inside an ALKIS area that shall not be entered, and an OSM bridge over a Berlin bridge deck are dropped when half (520: 80 %) of their area lies under the higher-ranked area, widened by 0.25 mm on paper. The kept object names the other in `freemapper:also`. An OSM memorial within 5 m of a cadastre monument and OSM kerbs and embankments along cadastre ones go too. Viktoriapark: 520 from 117 to 94, 522 from 211 to 192. Lübeck: 520 from 1 125 to 1 112, 522 from 280 to 221 (ADR 0078).
- Control sites in buildings no longer thin their neighbours. The 25 m rule now runs after the sites in buildings are dropped.

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
