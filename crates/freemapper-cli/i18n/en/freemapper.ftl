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

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; no area, cannot be a map area): { $place }
cli-find-hint = Create the project with: freemapper init <dir> --osm { $id }
cli-find-no-area = None of these places has an area that can be a map area. Draw the area with: freemapper select <dir>
cli-find-nothing = no place found for "{ $query }"; try another name, or draw the area with: freemapper select <dir>

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
cli-save-running = A run is going on for this project. Save again when it has finished.
cli-save-unknown-kind = Unknown map kind "{ $kind }". Use schoolyard or sprint.
cli-save-scale-too-large = The scale { $scale } is too large.
cli-save-bad-content = The content settings are not valid. Nothing was saved.

## The area page (`web/`, ADR 0096). The page gets every `page-*` and `suite-*` message through
## `api/texts`; a `{ $name }` stays in the text and the page fills it in. No selectors: a count has
## two messages, `<id>-one` and `<id>-other`. AGENTS.md, the manual and scripts/browser-smoke.md quote
## these words.

page-title = freemapper – area selection
page-heading = Choose the map area
page-update-available = freemapper { $version } is available. Update with
page-update-notes = Release notes
page-folder = Project folder
page-pick = Search a place
page-query-placeholder = e.g. Clara-Zetkin-Schule Strausberg
page-search-button = Search
page-searching = Searching …
page-no-results = Nothing found.
page-point-only = a point only, not an area
page-draw = Draw it yourself
page-rectangle = Rectangle
page-polygon = Polygon
page-finish-polygon = Close polygon
page-clear = Clear
page-hint-rectangle = Click two opposite corners on the map.
page-hint-polygon = Click the corners one after another. To close it, click the first corner, double-click or click "Close polygon". Esc cancels, Backspace removes the last corner.
page-hint-idle = Pick a place from the search or draw the area yourself.
page-summary = Area
page-source-osm = From OpenStreetMap: { $name }
page-source-drawn = Drawn by hand
page-size = Size with buffer: { $width } × { $height } m
page-tiles-one = { $n } tile: { $names }
page-tiles-other = { $n } tiles: { $names }
page-download = Download: about { $mb } MB
page-download-none = Download: everything is cached already
page-download-details = What is downloaded
page-cached = cached
page-disk = Disk space: { $volumes }
page-disk-volume = { $free } free, about { $needed } needed
page-cache-volume = cache
page-project-volume = project folder
page-no-room = does not fit on the disk
page-no-room-skip = Not everything fits on the disk (freemapper keeps { $reserve } free). The fetch leaves out the marked parts. Free some space first if you need them:
page-no-room-stop = Not enough disk space (freemapper keeps { $reserve } free). There is no map without the marked parts, so the fetch stops before it downloads anything. Free some space first:
page-disk-unknown = freemapper cannot read the free disk space ({ $error }). It fetches without the check.
page-too-large = The area is too large: at most { $max } m per side (with buffer). Please choose a smaller one.
page-outside = The area lies outside the supported states (Brandenburg, Berlin, Saxony, Saxony-Anhalt, Hamburg, Hesse, Schleswig-Holstein, Mecklenburg-Vorpommern, North Rhine-Westphalia, Lower Saxony, Rhineland-Palatinate); freemapper has no data sources for other regions yet.
page-settings = Map
page-kind = Map kind
page-schoolyard = Schoolyard map (1:1000)
page-sprint = Sprint map (1:4000)
page-scale = Scale 1:
page-map-title = Title
page-settings-fixed = The project exists already; map kind, scale and title are in freemapper.toml.
page-save = Save
page-saving = Saving …
page-saved = Saved to freemapper.toml.
page-zone-changed = Saved. The area now lies in another UTM zone; freemapper fetches its data again on the next run.
page-provider-changed = Saved. Another data provider serves the elevation and aerial imagery of the new area; freemapper fetches its data again on the next run.
page-save-stale = freemapper.toml was changed in another window or by another program since this page read it. Nothing was saved; the page now knows the saved state. Save again to replace it with the area shown here.
page-has-map = The existing map (.omap) stays as it is; freemapper makes new templates and candidates for the new area. The option "Replace the existing map" below (or "freemapper run --force") makes the map anew too and keeps the old one as .omap.bak.
page-done = Done
page-closed = Done. You can close this window.
page-next = Next: freemapper run "{ $dir }"
page-made-files = Created: { $files }. Folder: { $dir }
page-back-to-agent = Tell your AI assistant that the area is saved.
page-nothing-saved = Nothing was saved.
page-failed = Error: { $error }
page-layer-osm = OpenStreetMap
page-layer-aerial = Aerial photo Brandenburg/Berlin (LGB)
page-layer-aerial-saxony = Aerial photo Saxony (GeoSN)
page-layer-aerial-saxony-anhalt = Aerial photo Saxony-Anhalt (LVermGeo)
page-layer-aerial-hamburg = Aerial photo Hamburg (LGV)
page-layer-aerial-hessen = Aerial photo Hesse (HVBG)
page-layer-aerial-schleswig-holstein = Aerial photo Schleswig-Holstein (LVermGeo SH)
page-layer-aerial-mecklenburg-vorpommern = Aerial photo Mecklenburg-Vorpommern (LAiV M-V)
page-layer-aerial-nordrhein-westfalen = Aerial photo North Rhine-Westphalia (Geobasis NRW)
page-layer-aerial-niedersachsen = Aerial photo Lower Saxony (LGLN)
page-layer-aerial-rheinland-pfalz = Aerial photo Rhineland-Palatinate (LVermGeo RP)

## The run panel of the area page (ADR 0090)

page-content-heading = What goes onto the map
page-content-help = The profile decides what goes straight onto the map and what stays a candidate to check. A choice for a group beats the profile. It is saved with the area.
page-content-profile-legend = Profile
page-content-profile-safe = Safe
page-content-profile-safe-hint = Every object stays where the build puts it: buildings, areas, water and paths go onto the map; walls, fences, trees and the like stay candidates.
page-content-profile-balanced = Balanced
page-content-profile-balanced-hint = Objects of the cadastre, the registers and the street survey go onto the map, and for walls, fences, hedges, water, trees and paths those of OpenStreetMap too. Everything else stays where the build puts it.
page-content-profile-full = Full
page-content-profile-full-hint = Everything goes onto the map, as in the training map.
page-content-groups-legend = By group
page-content-groups-help = "As in the profile" follows the profile. What that means for a group depends on where each object comes from.
page-content-place-profile = As in the profile
page-content-place-map = Map
page-content-place-candidates = Candidate
page-content-place-off = Off
page-content-group-walls = Walls
page-content-group-fences = Fences
page-content-group-hedges = Hedges
page-content-group-slopes = Banks and cliffs
page-content-group-stairs = Stairs
page-content-group-trees = Trees and tree groups
page-content-group-water-features = Water, fountains and springs
page-content-group-monuments = Towers, monuments and power lines
page-content-group-furniture = Street furniture and lamps
page-content-group-passages = Passages
page-content-group-contours = Contours
page-content-group-out-of-bounds = Areas that shall not be entered
page-content-group-paths = Paths
page-content-group-streets = Streets and squares
page-run-heading = Create the map
page-opt-ocad = Write an OCAD starter project
page-help-ocad = For everyone who carries on in OCAD.
page-opt-pdf = Printable PDF
page-help-pdf = Map objects only, no candidates.
page-opt-pdf-candidates = Training PDF with every candidate
page-opt-scale-bar = Scale bar on the PDFs
page-help-scale-bar = German maps usually have none.
page-opt-sources = Source lines on the PDFs
page-sources-auto = automatic
page-sources-full = full
page-sources-credit-only = freemapper only
page-warn-credit-only = The PDFs then name freemapper only. The data licences (ODbL, dl-de/by-2-0) require attribution when you pass the map on; then you name the sources yourself. ATTRIBUTION.txt lists them all.
page-need-pdf = The scale bar and the source lines only apply when a PDF is made.
page-opt-offline = Cache only, no network
page-opt-force = Replace the existing map
page-help-force = The old file stays as .omap.bak.
page-run-start = Create map
page-run-again = Create again
page-run-cancel = Cancel
page-run-cancelling = Cancelling …
page-warnings-heading = Warnings
page-where-files = The files are in the project folder:
page-open-folder = Open folder
page-log-label = Log of the run
page-force-title = Replace the existing map?
page-force-text = The existing map is replaced. Your edits in it are not lost: the old file stays in the project folder as .omap.bak. Continue?
page-force-cancel = Cancel
page-force-confirm = Replace
page-step-fetch = Fetching data
page-step-derive = Computing templates
page-step-map = Writing the map
page-step-ocad = Writing the OCAD project
page-step-pdf = Writing the PDF
page-step-pdf-candidates = Writing the training PDF
page-step-line = Step { $index } of { $total }: { $label }
page-starting = The run is starting …
page-file-kind-map = map
page-file-kind-template = template
page-file-kind-ocad = OCAD map
page-file-kind-ocad-candidates = OCAD candidates
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = training PDF
page-file-line = { $path } ({ $kind }, { $size })
page-result-done = Done. The files are written.
page-result-failed = The run failed: { $error }
page-result-failed-kept = What is written stays in the project folder; a new run carries on from there.
page-result-cancelled = Cancelled. What is downloaded already stays in the cache; a new run carries on from there.
page-result-title-done = Map created
page-result-title-failed = Run failed
page-result-title-cancelled = Run cancelled
page-kept-map = The existing map was left as it is. "Replace the existing map" makes it anew and keeps the old one as .omap.bak.
page-no-files = No files were made.
page-opened-folder = The folder is open.
page-open-failed = { $message } Folder: { $dir }
page-why-no-area = Choose an area first.
page-why-checking = The preview is being computed …
page-why-too-large = The area is too large.
page-why-outside = The area lies outside the supported states.
page-why-disk = Not enough free disk space (see above).
page-why-running = A run is going on.
page-err-scale-bar-no-pdf = The scale bar needs a PDF.
page-err-sources-no-pdf = The source lines need a PDF.
page-save-failed = The area is not saved; no run was started.
page-poll-lost = No answer from the server. freemapper keeps trying.
page-run-unknown = The server no longer knows this run.

## The shared modules of the suite's pages (core ADR 0030)

suite-banner-dismiss = Dismiss

## update (spec 3.4 to 3.6)

update-source-build = This freemapper was built from source. Update it with git pull and cargo build --release --locked.
update-current = freemapper { $version } is up to date.
update-available = freemapper { $new } is available (you have { $old }), released { $released }.
update-details =
    Release notes: { $notes }
    Download: { $size } MB
update-decimal-separator = .
update-ask = Install it now? [y/N]
update-not-installed = Nothing was installed.
update-needs-yes = the input is not a terminal, so freemapper cannot ask before installing; run freemapper update --yes to install without asking
update-installing = downloading and checking freemapper { $version }
update-done = freemapper updated from { $old } to { $new }. Release notes: { $notes }
update-mcp-restart = A running MCP server keeps the old version until the AI assistant restarts it.
update-rolled-back = Went back to freemapper { $version }. The version you left is kept as { $path }.
update-not-writable = cannot write to the program folder { $path }, and freemapper does not ask for administrator rights. Unpack the new release there by hand, or move freemapper to a folder you can write to.
update-too-old = freemapper { $new } cannot be installed by the self-update of { $old }; install { $needed } or newer by hand, see { $notes }
update-busy = another update or rollback is running in the program folder (lock file { $path }); wait for it to finish and try again
update-bad-signature = the release could not be verified ({ $error }); nothing was installed
update-check-bad-signature = the newest freemapper release could not be verified ({ $error }); nothing was installed
update-no-manifest = the newest release cannot install itself (it has no update manifest); download it from { $page }
update-no-manifest-version = release { $version } does not exist or cannot install itself (it has no update manifest); see { $page }
update-no-asset = the release has no program for this system ({ $platform }); nothing was installed
update-no-previous = there is no previous version to go back to ({ $path } is missing)
update-failed = the update failed: { $error }
update-mode-set = Update mode set to { $mode } in { $path }.
update-mode-env = FREEMAPPER_UPDATE={ $value } overrides this setting while it is set.
update-mode-failed = could not write { $path }: { $error }
update-notice = freemapper { $new } is available (you have { $old }). Update with: freemapper update
update-applies-next-start = The new version applies from the next start of freemapper.
update-install-failed = freemapper { $new } was not installed: { $error }
update-no-config-dir = no configuration folder on this system; set FREEMAPPER_CONFIG_DIR
update-mcp-note = Note for the person: freemapper { $new } is available (this is { $old }). Ask them before calling install_update.
update-mcp-needs-confirm = nothing was installed; ask the person first and call install_update with confirm: true once they agree
update-mcp-note-manual = Note for the person: freemapper { $new } is available (this is { $old }), but install_update cannot install it; install { $needed } or newer by hand, see { $notes }
update-mcp-note-bad-signature = Note for the person: the newest freemapper release could not be verified ({ $error }); nothing was installed. Tell them, and do not install it some other way.
update-mcp-restart-now = Restart the freemapper MCP server now; it still runs the old version.

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
help-find = List the OpenStreetMap places for a name, areas first, with the id for init --osm
help-find-query = Name of the place, e.g. "Clara-Zetkin-Schule Strausberg"
help-find-json = Print the matches as JSON, the same fields as the MCP tool find_place
help-mcp = Serve the tools to an AI agent over MCP (stdin/stdout)
help-update = Install a newer freemapper from the GitHub releases, after asking. A build from source says how to update it instead
help-update-check = Only check: exit code 10 when a newer version exists, 0 when this one is current
help-update-yes = Install without asking; needed when the input is not a terminal
help-update-version = Install this version (x.y.z) instead of the newest one, also an older one
help-update-rollback = Go back to the version the last update replaced
help-update-mode = Store what the daily check does when it finds a newer version (default: notify). Works in every build, because it only writes the setting

## The summary of a run, for the MCP server and the area page

cli-internal-error = internal error: { $message }
cli-pdf-sources-invalid = pdf_sources must be auto, full or credit-only, not { $value }
cli-summary-wrote-map = Wrote the map { $path }
cli-summary-kept-map = Kept the existing map { $path } (it may contain the mapper's work)
cli-summary-map-lacks-templates = The map does not reference the image templates yet. Start the pipeline with force to replace it and keep the old one as .omap.bak.
cli-summary-map-lacks-buildings = The map has no ALKIS buildings yet. Start the pipeline with force to add them; it replaces the map and keeps the old one as .omap.bak.
cli-summary-map-lacks-canopies = The map lacks the ALKIS canopies and carports (522) that are now map objects. Start the pipeline with force to add them; it replaces the map and keeps the old one as .omap.bak.
cli-summary-wrote-ocad = Wrote the OCAD map { $path } ({ $count } objects)
cli-summary-kept-ocad = Kept the existing OCAD map { $path } (it may contain the mapper's work; force replaces it)
cli-summary-ocad-left-out = Left out of the OCAD map: { $skipped } objects and { $dropped } degenerate parts
cli-summary-ocad-symbols = In OCAD, choose Symbol > Load Colors and Symbols From… and pick the official ISSprOM 2019 or school symbol set; it replaces the placeholder symbols by number.
cli-summary-ocad-failed = OCAD export failed: { $error }
cli-summary-pdf-without-credits = PDF without source credits: ODbL and dl-de/by-2-0 ask you to name the sources when you pass the map on; see ATTRIBUTION.txt
cli-summary-wrote-pdf = Wrote the map { $path } (without candidates)
cli-summary-pdf-failed = PDF export failed: { $error }
cli-summary-wrote-pdf-candidates = Wrote the training map { $path } (every candidate drawn as a map object)
cli-summary-pdf-candidates-failed = Candidates PDF export failed: { $error }

## The MCP server. The descriptions of the tools and parameters equal the texts in `src/mcp.rs`; a test compares them.

mcp-error-no-home = no home folder on this system; use an absolute path
mcp-error-relative-path = { $path } is a relative path; use an absolute path or one starting with ~
mcp-error-home-folder = { $path } is the home folder itself; use a folder inside it, e.g. ~/maps/school
mcp-error-no-project = no freemapper project in { $path }; use create_project first
mcp-error-unknown-kind = unknown kind { $kind }; use "schoolyard" or "sprint"
mcp-error-reading = reading { $path }: { $error }
mcp-error-one-area-source = give exactly one of osm_id (from find_place) or geojson
mcp-error-page-open = the area selection page for this project is still open; ask the user to save the area and press Done on the page, then start the pipeline
mcp-error-no-job = no job { $job }; start_pipeline returns the job ids of this session
mcp-error-job-running = { $job } is still running for { $path }; one pipeline runs at a time because projects share the download cache. Start this one when project_status reports finished or failed
mcp-error-page-start = cannot start the page: { $error }
mcp-created-project = Created project { $path } for { $area } ({ $kind }, 1:{ $scale }). Next: estimate_download, then start_pipeline.
mcp-created-project-given-area = Created project { $path } for the given area ({ $kind }, 1:{ $scale }). Next: estimate_download, then start_pipeline.
mcp-page-already-open = The page for this folder is already open; it keeps the settings it was opened with.
mcp-page-next = Ask the user to choose the area on the page, press Save and then Done.
mcp-preview-of = Preview of { $layer }
mcp-tool-check-update = Whether a newer freemapper release exists: its version, release date, release notes and download size, or that this one is up to date. Installs nothing.
mcp-tool-create-project = Create a project folder for a map area. Give osm_id (from find_place) or a GeoJSON file. kind is schoolyard (1:1000, default) or sprint (1:4000).
mcp-param-create-project-dir = New project folder (absolute path or starting with ~).
mcp-param-create-project-geojson = Instead of osm_id: a GeoJSON file with the map area polygon (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (default, 1:1000) or "sprint" (1:4000).
mcp-param-create-project-osm-id = OSM object of the map area from find_place, e.g. "way/992423119".
mcp-param-create-project-part = With osm_id: which part of an area of several separate parts to use (1 is the largest).
mcp-param-create-project-scale = Scale denominator: schoolyard 1000 (default); sprint 4000 (default) or 5000. Other values are refused; for 1:3000 draw at 4000, freemapper-print enlarges when printing.
mcp-param-create-project-title = Map title for the peripheral information (default: the folder name).
mcp-tool-estimate-download = What start_pipeline will download for a project: the tiles, each download with its size in MB, whether it is cached and whether it fits on the disk, the MB still to download, and under disk the free space of the cache and project volumes with messages for what does not fit (tell the user).
mcp-param-estimate-download-dir = Project folder (absolute path or starting with ~).
mcp-tool-find-place = Search a school, park or other place by name (OpenStreetMap Nominatim). Returns up to 5 matches, areas first; only matches with has_area can be a map area. An empty list means nothing was found.
mcp-param-find-place-query = Name of the school or place, e.g. "Clara-Zetkin-Schule Strausberg".
mcp-tool-install-update = Install the newest freemapper release over this program. Call it only after the person agreed, with confirm true. The new version applies once the assistant restarts the freemapper server.
mcp-param-install-update-confirm = true once the person agreed to install the newer version; without it nothing is installed.
mcp-tool-list-layers = The layers of a project: the map, the candidates and the raster templates, each with its file, whether it is made yet and which step makes it.
mcp-param-list-layers-dir = Project folder (absolute path or starting with ~).
mcp-tool-open-area-selector = Open a page in the browser where the user draws the map area (rectangle or polygon) or picks a place, sees the tiles and the download size, and saves it; saving creates the project folder or changes its area. Returns the URL to give the user. The page runs until the user presses Done; calling again for the same folder returns the same URL.
mcp-param-open-area-selector-dir = Project folder (absolute path or starting with ~); created when the user saves.
mcp-param-open-area-selector-kind = Map kind of a new project: "schoolyard" (default, 1:1000) or "sprint" (1:4000).
mcp-param-open-area-selector-ocad = The run panel starts with the OCAD starter project ticked. Default false.
mcp-param-open-area-selector-offline = The run panel starts with "cache only, no network" ticked. Default false.
mcp-param-open-area-selector-open-browser = Also open the page in the default browser of this computer. Default true.
mcp-param-open-area-selector-pdf = The run panel starts with the printable PDF ticked. Default false.
mcp-param-open-area-selector-pdf-candidates = The run panel starts with the training PDF (every candidate) ticked. Default false.
mcp-param-open-area-selector-pdf-scale-bar = The run panel starts with the scale bar on the PDFs ticked. Default false.
mcp-param-open-area-selector-pdf-sources = Source lines on the PDFs the panel starts with: "auto" (default), "full" or "credit-only", as for start_pipeline.
mcp-param-open-area-selector-scale = Scale denominator of a new project: schoolyard 1000 (default); sprint 4000 (default) or 5000. Other values are refused.
mcp-param-open-area-selector-title = Map title of a new project.
mcp-tool-preview-layer = A PNG preview (at most 800 px) of a raster template: orthophoto, hillshade, slope or ndsm.
mcp-param-preview-layer-dir = Project folder (absolute path or starting with ~).
mcp-param-preview-layer-layer = "orthophoto", "hillshade", "slope" or "ndsm".
mcp-tool-project-status = State of a pipeline job: running, finished or failed, the last progress lines, warnings, and the result or the error.
mcp-param-project-status-job-id = Job id from start_pipeline, e.g. "job-1".
mcp-tool-start-pipeline = Download the data, compute the templates and write the starter project (fetch, derive, export). Runs in the background and returns a job_id; poll project_status. One pipeline runs at a time. It keeps an existing map unless force is true.
mcp-param-start-pipeline-dir = Project folder (absolute path or starting with ~).
mcp-param-start-pipeline-force = Replace an existing map file and keep the old one as .omap.bak. Default false.
mcp-param-start-pipeline-ocad = Also write an OCAD starting point into the folder ocad/ (OCAD 12 format, for users who edit in OCAD). Default false.
mcp-param-start-pipeline-offline = Use the download cache only, without network access. Default false.
mcp-param-start-pipeline-pdf = Also write <name>.pdf, the map ready to print (the map objects only, no candidates). Default false.
mcp-param-start-pipeline-pdf-candidates = Also write <name>-candidates.pdf, the training map (the map plus every candidate drawn as a map object; check them in the terrain). Default false.
mcp-param-start-pipeline-pdf-scale-bar = Draw the scale bar on the PDFs. Default false (German maps usually have none; the scale number stays in the title block).
mcp-param-start-pipeline-pdf-sources = Source lines on the PDFs: "auto" (default; the full list, shortened only when it takes more than 5 lines), "full" (never shortened) or "credit-only" (only the freemapper credit line; ODbL and dl-de/by-2-0 then require naming the sources when the map is passed on). ATTRIBUTION.txt always lists every source.
mcp-instructions = freemapper makes OpenOrienteering Mapper starter projects for orienteering maps (schoolyard 1:1000, sprint 1:4000) from official geodata of Brandenburg, Berlin, Saxony, Saxony-Anhalt, Hamburg, Hesse, Schleswig-Holstein, Mecklenburg-Vorpommern, North Rhine-Westphalia, Lower Saxony and Rhineland-Palatinate, and from OSM. Workflow: find_place (search the school or area by name; pick a result with has_area) → create_project (a new folder, absolute path, with the osm_id) → estimate_download (tell the user how much will be downloaded) → start_pipeline (returns a job id; downloading can take minutes) → project_status (poll every few seconds until the state is finished or failed) → list_layers and preview_layer (show the user the templates). The user then opens the map file in OpenOrienteering Mapper. When no place fits or the user wants to draw the area, open_area_selector opens a page in the browser where the user draws or picks the area and saves it (this creates the project); continue with estimate_download once the user says it is saved. Paths must be absolute or start with ~. One pipeline runs at a time. check_update tells whether a newer freemapper exists; install_update installs it, only with confirm: true after the person agreed, and the assistant then restarts the freemapper server.

cli-content-saved = Content setting saved in freemapper.toml: { $summary }
cli-content-invalid = Cannot read the content setting: { $detail }
help-arg-content = What goes straight onto the map: `safe` leaves every object where the build puts it, `balanced` (default) puts the objects of the cadastre and the registers on the map, `full` puts everything on the map. The choice is saved in freemapper.toml
help-arg-content-set = Send a group of objects to `map`, `candidates` or `off`, as `GROUP=PLACE`, e.g. `walls=map`; may be repeated. Groups: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, out-of-bounds, paths, streets. Saved in freemapper.toml
mcp-param-create-project-content = What goes straight onto the map: "safe", "balanced" (default) or "full". Saved in the project. balanced puts the objects of the cadastre and the registers on the map, full puts everything on it.
mcp-param-create-project-content-set = Groups sent somewhere else than the profile says, as "group=place" with place map, candidates or off, e.g. "walls=map". Groups: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, out-of-bounds, paths, streets.
mcp-param-start-pipeline-content = Change what goes straight onto the map before the run: "safe", "balanced" or "full". Saved in the project. Default: the project's setting, else balanced.
mcp-param-start-pipeline-content-set = Groups sent somewhere else than the profile says, as "group=place" with place map, candidates or off, e.g. "fences=candidates". Saved in the project.
mcp-error-content = Cannot read the content setting: { $detail }
