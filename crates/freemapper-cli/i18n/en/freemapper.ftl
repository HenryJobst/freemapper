# Program texts of freemapper, English (ADR 0092). English is the reference: every other language
# has the same message ids and the same variables. Texts from other programs (operating system
# errors, parser messages) arrive in variables and stay as they are.

## The --lang switch

cli-lang-unknown = { $value } is not a language freemapper has texts for; available: { $available }

## init

cli-part-needs-osm = --part picks a part of an OSM area and needs --osm; with --place, look up the id first and use --osm <id> --part <n>
cli-reading-file = reading { $path }
cli-no-area-for-place = no area found for "{ $query }". If the place is an area of several separate parts, pick one with --osm <id> --part <n>, or use --geojson
cli-using-place = Using { $name } ({ $id })
cli-created-project = Created project { $path }

## Cache

cli-no-cache-dir = no cache directory on this system; pass --cache-dir

## fetch, derive, run, export

cli-fetched-tiles = { $count ->
        [one] Fetched 1 tile
       *[other] Fetched { $count } tiles
    }
cli-wrote = Wrote { $path }
cli-kept-map = Kept { $path } (it may contain your work)
cli-map-lacks-templates = The map does not reference the image templates yet. Add them in OOM (Templates > Open template…) or replace the map with --force, which keeps the old one as .omap.bak.
cli-map-lacks-buildings = The map has no ALKIS buildings yet. To add them, replace it with --force, which keeps the old one as .omap.bak.
cli-map-lacks-canopies = The map lacks the ALKIS canopies and carports (522) that are now map objects. To add them, run with --force, which keeps the old map as .omap.bak.

## PDF

cli-pdf-without-credits = PDF without source credits: ODbL and dl-de/by-2-0 ask you to name the sources when you pass the map on; see ATTRIBUTION.txt
cli-wrote-pdf = Wrote { $path } (the map without candidates)
cli-wrote-pdf-candidates = Wrote { $path } (training map with every candidate drawn as a map object; check it in the terrain)

## OCAD

cli-ocad-kept = Kept { $path } (it may contain your work; --force replaces it and keeps the old one as .ocd.bak)
cli-ocad-wrote = { $count ->
        [one] Wrote { $path } (1 object)
       *[other] Wrote { $path } ({ $count } objects)
    }
cli-ocad-left-out = Left out of the OCAD map: { $skipped } objects (no symbol for OCAD, no area, or outside OCAD's drawing area of ±2 m) and { $dropped } degenerate line parts or holes.
cli-ocad-load-symbols = In OCAD, choose Symbol > Load Colors and Symbols From… and pick the official ISSprOM 2019 or school symbol set; it replaces freemapper's placeholder symbols by number.

## select

cli-select-running = Area selection for { $path }: press "Done" on the page or Ctrl+C to stop.
cli-select-no-browser = Could not open the browser ({ $error }); open the address above.

## Folder lock

cli-lock-held = another freemapper run is using this folder (since { $time }, started by { $who }). Wait for it or stop it first.
cli-lock-who-cli = the terminal
cli-lock-who-mcp = an AI agent (MCP)
cli-lock-who-page = the selection page
cli-lock-who-unknown = another program
cli-lock-failed = could not lock the project folder for this run: { $error }
cli-lock-stuck = something other than a lock file is in the way and could not be removed

## Run routes of the selection page

cli-run-bad-request = The run options are not valid: { $error }
cli-run-no-project = There is no saved project in this folder yet. Save the area first.
cli-run-already-running = A run ({ $job }) is already going on for this project. Wait for it to end or cancel it first.
cli-run-disk-stops = Not enough free disk space for the download: { $messages }
cli-run-busy-elsewhere = Another project of this program is running a pipeline right now. Projects share the download cache, so only one run goes at a time. Try again when it has finished.
cli-run-nothing-to-cancel = No run is going on, so there is nothing to cancel.
cli-run-unknown-job = There is no run { $job } for this project.
cli-status-too-fast = Status requests come too often. Ask at most five times a second.
cli-open-folder-failed = The folder could not be opened ({ $error }).
cli-open-folder-no-opener = This computer has no program to open folders ({ $program }). Open the folder by hand.
cli-open-folder-missing = The project folder does not exist yet. Save the area first.
cli-open-folder-refused = The folder name is not safe to pass on to the file manager.

## Errors. In English each text equals the message of the library's error type, which stays
## English for logs and tests. Parts that other programs word (the operating system, parsers,
## servers) arrive in variables and stay as they are.

cli-error-prefix = error
error-cancelled = cancelled
cli-cancelled-kept = cancelled; kept: { $files }
error-io = { $path }: { $message }
error-io-disk-full = the disk is full, so freemapper could not write { $path }. Free some space and try again

## Errors: project

error-project-already-exists = project already exists: { $path }
error-project-map-exists = map already exists (it may contain your work); use --force to replace it: { $path }
error-project-invalid-scale = invalid scale 1:{ $scale }
error-project-scale-not-allowed = scale 1:{ $scale } is not allowed for { $kind } maps; use { $allowed }
error-project-scale-not-allowed-enlarge = scale 1:{ $scale } is not allowed for { $kind } maps; use { $allowed }. Draw at 1:4000, freemapper-print enlarges to 1:3000 when printing
word-kind-schoolyard = schoolyard
word-kind-sprint = sprint
error-project-toml = invalid project file: { $message }
error-project-not-fetched = no tiles fetched yet; run `freemapper fetch` first
error-project-ocd = OCAD export: { $message }
error-project-template-outdated = { $path }: image and world file do not match the map area; run `freemapper derive` again
error-project-tiles-outdated = the map area needs tiles { $tiles } that were not fetched; run `freemapper fetch`
error-project-buildings-not-fetched = no ALKIS buildings stored for this map area; run `freemapper fetch` online first
error-project-orthophoto-not-fetched = no orthophoto stored for this map area; run `freemapper fetch` online first
error-project-terrain-not-fetched = no terrain model stored for this map area; run `freemapper fetch` online first
error-project-candidates-outdated = the stored OSM candidates do not cover the map area; run `freemapper fetch`
error-project-stored = { $path }: { $message }; run `freemapper fetch` again
error-project-area-too-large = map area plus buffer is { $width } × { $height } m, more than { $limit } m per side; reduce the map area or buffer_m in freemapper.toml
error-project-outside-coverage = the map area lies outside the supported states (Brandenburg, Berlin, Saxony, Saxony-Anhalt, Hamburg, Hesse, Schleswig-Holstein, Mecklenburg-Vorpommern, North Rhine-Westphalia, Lower Saxony, Rhineland-Palatinate); freemapper has data sources for those only so far

## Errors: disk space

error-space-required = not enough disk space for { $part }. It needs { $needed }, but { $path } has only { $left } free beyond the { $reserve } reserve. Free some space and run `freemapper fetch` again
error-space-skipped = freemapper skips { $part } for lack of disk space. It needs { $needed }, but { $path } has only { $left } free beyond the { $reserve } reserve. Free some space and run `freemapper fetch` again
word-part-map-data = the ALKIS and OSM map data
word-part-terrain = the terrain model
word-part-orthophoto = the orthophoto
word-part-surface = the surface model for the nDSM template
word-part-osm-update = the update of the cached OSM extract
word-part-k-five = the Berlin K5 map template
word-part-berlin-layers = the Berlin ALKIS structures, tree cadastre, street survey and point layers
word-part-state-layers = the full ALKIS structures and the tree cadastres

## Errors: downloads

error-fetch-http = download of { $url } failed: { $message }
error-fetch-corrupt-archive = download of { $url } is not a complete tile archive: { $message }
error-fetch-corrupt-extract = download of { $url } is not a complete OSM extract: { $message }
error-fetch-tiles-not-covered = { $count ->
        [one] tile { $tiles } is not covered by the published tiles; reduce the map area or buffer_m in freemapper.toml
       *[other] tiles { $tiles } are not covered by the published tiles; reduce the map area or buffer_m in freemapper.toml
    }
error-fetch-missing-offline = { $count ->
        [one] offline and not cached: { $product } tile { $tiles }; run `freemapper fetch` online
       *[other] offline and not cached: { $product } tiles { $tiles }; run `freemapper fetch` online
    }
word-product-orthophoto = orthophoto (DOP20)
word-product-dtm = terrain model (DGM1)
word-product-image-dsm = surface model (bDOM)
word-product-saxony-dtm = terrain model (GeoSN DGM1)
word-product-saxony-dsm = surface model (GeoSN DOM1)
error-fetch-osm-extract-missing = no OSM extract cached; run `freemapper fetch` online
error-fetch-not-found = { $name } not found on the server
error-fetch-refused = { $url }: the server refused the request (HTTP { $status })
error-fetch-refused-detail = { $url }: the server refused the request (HTTP { $status }): { $detail }
error-fetch-too-large = download of { $url } is larger than { $limit } MB; stopped
error-fetch-not-stored = no { $label } stored for this map area: { $path } is missing, cannot be read or does not cover it; run `freemapper fetch` online first
error-fetch-range-ignored = { $url }: the server ignored the byte range { $range } (HTTP { $status })
error-fetch-currency-csv = currency CSV line { $line }: { $message }
error-alkis = invalid ALKIS data: { $message }
error-osm = OSM data: { $message }

## Errors: templates and rasters

error-derive-zip = tile archive: { $message }
error-derive-image = image: { $message }
error-derive-no-tiles = no tiles to derive from
error-derive-area-too-large = map area too large for one template: { $width } × { $height } pixels (at most { $limit } per side); reduce the map area or buffer_m in freemapper.toml
error-raster-world-file = invalid world file: { $message }
error-raster-misaligned = rasters are not aligned to the same pixel grid: { $message }
error-raster-pixel-size = rasters have different pixel sizes: { $message }

## Errors: map area, georeferencing, map file, OCAD file

error-area-geojson = invalid GeoJSON: { $message }
error-area-no-polygon = GeoJSON contains no polygon
error-area-multi-polygon = MultiPolygon with { $parts } parts; split the map area into one polygon
error-area-out-of-range = coordinate out of range: lat { $lat }, lon { $lon }
error-area-not-an-area = not a usable map area: { $message }
error-georef-date = date { $date } is outside the World Magnetic Model validity
error-georef-map-range = point E { $easting } N { $northing } lies beyond the map coordinate range
error-omap-xml = invalid OOM XML: { $message }
error-omap-missing-element = element <{ $name }> not found
error-ocd-duplicate-symbol = symbol number { $symbol } is used twice
error-ocd-unknown-symbol = object refers to symbol { $symbol }, which is not in the file
error-ocd-wrong-type = an object of symbol { $symbol } has the wrong type for its symbol

## The help (`--help`). The English texts equal the doc comments in `src/main.rs`; a test compares them.

cli-heading-usage = Usage:
cli-heading-commands = Commands:
cli-heading-options = Options:
cli-heading-arguments = Arguments:
cli-help-possible-values = possible values
cli-help-default = default
cli-arg-help = Print help
cli-arg-version = Print version
cli-help-help = Show the help of a command
cli-arg-help-command = The command to show the help of, for example run
cli-parse-hint = For more information, try '--help'.
cli-parse-unknown-argument = unexpected argument { $arg }
cli-parse-missing-required = missing required argument: { $args }
cli-parse-invalid-value = invalid value { $value } for { $arg }
cli-parse-invalid-value-choices = invalid value { $value } for { $arg }; possible values: { $values }
cli-parse-invalid-subcommand = unknown command { $value }
cli-parse-missing-subcommand = a command is required
cli-parse-too-many-values = unexpected value { $value } for { $arg }
cli-parse-argument-conflict = { $arg } cannot be used with { $other }

help-freemapper = Builds OOM starter projects from geodata
help-freemapper-lang = Language of the program texts, `de` or `en` (default: FREEMAPPER_LANG, then the system language, then English)
help-init = Create a project folder for a map area
help-init-dir = Project folder to create
help-init-geojson = GeoJSON file with the map area polygon (EPSG:4326)
help-init-osm = OSM object of the map area, e.g. way/992423119
help-init-place = Name of the place, e.g. "Clara-Zetkin-Schule Strausberg"; freemapper uses the best match with an area
help-init-part = Which part of an OSM area of several separate parts to use (1 is the largest)
help-init-kind = Map kind
help-arg-scale = Scale denominator of a new project: schoolyard 1000 (default); sprint 4000 (default) or 5000. Other values are refused, because freemapper-print refuses them
help-init-title = Map title for the peripheral information (default: the folder name)
help-export = Write the starter project (OOM map, templates, attribution) into a project folder
help-export-dir = Project folder
help-arg-force = Replace an existing map file and keep the old one as `<name>.omap.bak`
help-export-ocad = Also write an OCAD starting point into `ocad/` (OCAD 12 format); an existing OOM map then stays unless you pass --force
help-export-pdf = Also write `<name>.pdf`, the printable map (the map objects only, no candidates); an existing OOM map then stays unless you pass --force
help-export-pdf-candidates = Also write `<name>-candidates.pdf`, the training map (the map plus every candidate drawn as a map object); an existing OOM map then stays unless you pass --force
help-export-pdf-scale-bar = Draw the scale bar on the PDFs (off by default, German maps usually have none; the scale number stays in the title block)
help-export-pdf-sources = Source lines on the PDFs: `auto` prints the full list and shortens it only when it would take more than 5 lines, `full` never shortens, `credit-only` prints only the freemapper credit line (ODbL and dl-de/by-2-0 then ask you to name the sources when you pass the map on). ATTRIBUTION.txt always lists every source
help-fetch = Download the data for the map area (orthophoto, terrain model, nDSM, ALKIS buildings, OSM)
help-fetch-dir = Project folder
help-fetch-offline = Use the cache only, without network access
help-arg-cache-dir = Cache folder for downloaded tiles (default: the system cache folder + `freemapper`)
help-derive = Compute the templates (orthophoto, hillshade, slope, nDSM) and contours from the fetched data
help-derive-dir = Project folder
help-run = Fetch, derive and export in one go
help-run-dir = Project folder
help-run-offline = Use the cache only, without network access
help-run-ocad = Also write an OCAD starting point into `ocad/` (OCAD 12 format)
help-run-pdf = Also write `<name>.pdf`, the printable map (the map objects only, no candidates)
help-run-pdf-candidates = Also write `<name>-candidates.pdf`, the training map (the map plus every candidate)
help-run-pdf-scale-bar = Draw the scale bar on the PDFs (off by default; the scale number stays in the title block)
help-run-pdf-sources = Source lines on the PDFs: `auto` prints the full list and shortens it only when it would take more than 5 lines, `full` never shortens, `credit-only` prints only the freemapper credit line (ODbL and dl-de/by-2-0 then ask you to name the sources when you pass the map on). ATTRIBUTION.txt always lists every source
help-select = Choose the map area in the browser by drawing it or picking a place. Saving writes freemapper.toml and creates the project folder if needed
help-select-dir = Project folder (created on saving if it does not exist)
help-select-kind = Map kind of a new project
help-select-title = Map title of a new project
help-select-no-browser = Only print the address of the page; do not open the browser
help-select-port = Port on 127.0.0.1 (default: a free one)
help-select-offline = The page starts with "cache only, no network" ticked
help-select-ocad = The page starts with the OCAD starter project ticked
help-select-pdf = The page starts with the printable PDF ticked
help-select-pdf-candidates = The page starts with the training PDF (every candidate) ticked
help-select-pdf-scale-bar = The page starts with the scale bar on the PDFs ticked
help-select-pdf-sources = The page starts with this choice of source lines on the PDFs, as for `run`
help-mcp = Serve the tools to an AI agent over MCP (stdin/stdout)
