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
cli-ocad-load-symbols = In OCAD, choose Symbol > Load Colors and Symbols From… and pick the official symbol set{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , the IOF school set 2019
       *[other] {" "}ISSprOM 2019
    }; it replaces freemapper's placeholder symbols by number.

## select

cli-select-running = Area selection for { $path }: press "Done" on the page or Ctrl+C to stop.
cli-select-no-browser = Could not open the browser ({ $error }); open the address above.

## find

cli-find-place = { $id }  { $name } ({ $category }={ $kind }): { $place }
cli-find-place-no-area = { $id }  { $name } ({ $category }={ $kind }; no area, cannot be a map area): { $place }
cli-find-hint = Create the project with: freemapper init <dir> --osm { $id }
cli-find-no-area = None of these places has an area that can be a map area. Draw the area with: freemapper select <dir>
cli-find-nothing = no place found for "{ $query }"; try another name, or draw the area with: freemapper select <dir>

## adapters (ADR 0082). A finding of a manifest arrives in `{ $finding }` with its file, line, key
## and code, in English.

cli-adapter-skipped = warning: the adapter in { $folder } is left out: { $finding }
cli-adapter-note = warning: the adapter in { $folder } loads, but: { $finding }
cli-adapter-from-project = warning: the adapter { $adapter } from the project folder is in use; it may ask these hosts: { $hosts }
cli-adapter-overlaps = warning: the adapter { $adapter } also covers ground of { $others }; its objects go onto maps there
cli-adapter-dir-relative = warning: FREEMAPPER_ADAPTER_DIR is not an absolute path ({ $path }); it is ignored
cli-adapters-no-project = warning: { $path } is not a folder; no adapters of a project were read
cli-adapters-overlaps = shares ground with: { $others }
cli-adapters-line = { $adapter }  { $name }  { $source }  revision { $revision }  checked { $checked }  hosts: { $hosts }
cli-adapters-source-built-in = built in
cli-adapters-source-user = from your folder
cli-adapters-source-project = from the project
cli-adapters-never-checked = never
cli-adapters-user-dir = Your adapters are read from { $path }
cli-adapters-no-user-dir = This system has no folder for your adapters; FREEMAPPER_ADAPTER_DIR names one

## adapters allow-host (ADR 0110)

cli-allow-host-added = freemapper may now also ask { $host } for { $name }. The entry is in { $file }; `freemapper adapters disallow-host { $adapter } { $host }` takes it back. An area page or an AI assistant that is running reads the entry when it is started again
cli-allow-host-already = { $host } is already allowed for { $name }
cli-allow-host-declared = the adapter of { $name } names { $host } itself; nothing to allow
cli-allow-host-refused = { $host } cannot be allowed: { $reason }
cli-allow-host-unknown-adapter = there is no built-in adapter { $adapter }; the built-in adapters are: { $adapters }
cli-allow-host-folder-adapter = { $adapter } is an adapter of your adapter folder; add the host to `[net] hosts` of its manifest instead
cli-allow-host-no-config = this system has no configuration folder to keep allowed hosts in
cli-allow-host-write = { $file } could not be written: { $detail }
cli-allow-host-unreadable = { $file } cannot be read ({ $detail }); correct or remove the file first; nothing was changed
cli-disallow-host-removed = freemapper no longer asks { $host } for { $name }
cli-disallow-host-absent = { $host } was not allowed for { $name }; nothing to take back
cli-allowed-hosts-unreadable = warning: { $file } could not be read ({ $detail }); no extra hosts are allowed
cli-allowed-hosts-ignored = warning: { $file }: the entry { $host } for { $adapter } is ignored: { $reason }
cli-allowed-hosts-no-adapter = freemapper has no built-in adapter of that name
cli-allowed-host-redundant = warning: { $host } is now one of the hosts the adapter of { $name } names; your entry in { $file } was removed
cli-adapters-allowed = allowed by you: { $hosts }

## adapters check. A finding arrives in `{ $finding }` as in the lines above. `{ $capability }` is
## a fixed word (`rasters`, `alkis-buildings`, `nas`, `osm`), the same in every language.

cli-check-adapter = adapter { $adapter } ({ $name }), revision { $revision }, hosts: { $hosts }
cli-check-error = error: { $finding }
cli-check-warning = warning: { $finding }
cli-check-passed = { $folder } passes the check
cli-check-passed-warnings = { $folder } passes the check, with { $count ->
        [one] one warning
       *[other] { $count } warnings
    }
cli-check-failed = { $folder } fails the check
cli-check-no-folder = there is no folder { $folder }
cli-check-live-ok = { $capability } ok: { $detail }
cli-check-live-failed = { $capability } FAILED: { $detail }
cli-check-live-not-asked = { $capability } not asked: { $detail }
cli-check-live-box-outside = the box does not lie inside the boundary of the adapter { $adapter }; a live check asks the services for a place the adapter covers
cli-check-live-cannot = the live check cannot run: { $detail }
cli-check-detail-not-declared = the adapter declares none
cli-check-detail-stopped = an earlier part failed
cli-check-detail-see-warnings = see the warnings
cli-check-detail-outside-dataset = the box lies outside the dataset
cli-check-detail-objects = { $count ->
        [one] one object
       *[other] { $count } objects
    }
cli-check-detail-layers = { $layers } layers, { $count } objects
cli-check-detail-trees = { $count ->
        [one] one tree
       *[other] { $count } trees
    }
cli-check-detail-extract = { $label }, data of { $date }
cli-check-raster-orthophoto = orthophoto
cli-check-raster-terrain = terrain model
cli-check-raster-surface = surface model
cli-check-raster-tiles = { $product }: { $count ->
        [one] one tile
       *[other] { $count } tiles
    }

## Lists in sentences

# A list in a sentence: its last item after the others, which are joined by commas.
cli-list-and = { $first } and { $last }

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
cli-save-unknown-kind = Unknown map kind "{ $kind }". Use schoolyard, sprint or forest.
cli-save-scale-too-large = The scale { $scale } is too large.
cli-save-bad-content = The content settings are not valid. Nothing was saved.
cli-map-image-none = Create the map first.
cli-map-image-gone = This preview is no longer available.
cli-map-image-running = A run is going on for this project. The preview can be computed once it has finished.
cli-map-image-stale = The saved project no longer matches the generated map, for example because the area, map kind or scale changed. Start a run that replaces the map first ("Replace the existing map" on the page, or freemapper run --force).
cli-map-image-superseded = A newer preview was asked for meanwhile, so this one was not computed.
cli-map-image-bad-content = The content settings are not valid.
cli-map-image-bad-request = The request for a preview is not valid. Send a JSON object, with the content settings if any.

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
page-outside = The area lies outside the supported states ({ $states }); freemapper has no data sources for other regions yet.
page-outside-unknown = The area lies outside the regions freemapper supports; it has no data sources there yet.
page-adapters-failed = The aerial photos cannot be listed ({ $detail }). The map shows OpenStreetMap only; everything else works.
page-settings = Map
page-kind = Map kind
page-schoolyard = Schoolyard map (1:1000)
page-sprint = Sprint map (1:4000)
page-forest = Forest map (1:15000)
page-scale = Scale 1:
page-map-title = Title
page-contour-interval = Contour interval
page-contour-auto = automatic
page-settings-fixed = The project exists already; map kind, scale and title are in freemapper.toml. The contour interval can still be changed.
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
page-layer-aerial-of = Aerial photo { $state } ({ $agency })

## The "Create the map" block of the area page (ADR 0090)

page-content-heading = What goes onto the map
page-content-help = The profile decides what goes straight onto the map and what stays a candidate to check. A choice for a group beats the profile. It is saved with the area.
page-content-profile-legend = Profile
page-content-profile-safe = Safe
page-content-profile-safe-hint = Every object stays where the build puts it: buildings and the areas of the cadastre go onto the map; walls, fences, trees and much from OpenStreetMap (ponds, paths without a known surface) stay candidates.
page-content-profile-balanced = Balanced
page-content-profile-balanced-hint = Objects of the cadastre, the registers and the street survey go onto the map, with the contours and the areas that shall not be entered along railway tracks. Walls, fences, hedges, banks, stairs, water, trees, monuments, passages, paths and railways also come from OpenStreetMap. Street furniture, the other areas that shall not be entered and the forms from the terrain model stay candidates.
page-content-profile-full = Full
page-content-profile-full-hint = Everything goes onto the map, as in the training map; only hidden candidates stay out.
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
page-content-group-landforms = Knolls, depressions, banks and cliffs from the terrain model
page-content-group-vegetation = Vegetation from laser point clouds (Brandenburg, Berlin, North Rhine-Westphalia, Rhineland-Palatinate and Saarland; off until you choose it; downloads about 40 to 500 MB per km²)
page-content-group-out-of-bounds = Areas that shall not be entered
page-content-group-paths = Paths and railways
page-content-group-streets = Streets and squares
page-run-heading = Create the map
page-preview-heading = Map preview
page-preview-stored = As generated last.
page-preview-live = With the settings on this page, not saved yet.
page-preview-live-saved = With the saved settings, not generated in a run yet.
page-preview-computing = Computing …
page-preview-none = No map generated yet.
page-preview-later = This map has no preview yet. It comes with the next run that writes the map.
page-preview-stale = The area, map kind or scale changed since the map was generated. The image still shows the map as generated last; a new preview needs a run with "Replace the existing map".
page-preview-area-unsaved = The area on this page is not saved yet. The image shows the map as generated last; the new area needs a run with "Replace the existing map".
page-preview-run = As generated last. The page updates the image when the run ends.
page-preview-open = Open full size
page-preview-alt = Preview of the map objects
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
page-opt-permits = Permit notes
page-help-permits = Administrative units, protected areas and whom to ask, in permits.md. A hint, not legal advice.
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
page-step-permits = Writing the permit notes
page-step-line = Step { $index } of { $total }: { $label }
page-starting = The run is starting …
page-file-kind-map = map
page-file-kind-template = template
page-file-kind-ocad = OCAD map
page-file-kind-ocad-candidates = OCAD candidates
page-file-kind-pdf = PDF
page-file-kind-pdf-candidates = training PDF
page-file-kind-preview = map preview
page-file-kind-permits = permit notes
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
mcp-adapter-note = Note for the person: at its start the freemapper server found this about the adapter folders. Pass it on; the person fixes the folder or removes it, then restarts the server.
    { $lines }
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
error-project-invalid-contour-interval = invalid contour interval { $interval } m in freemapper.toml; it must be more than 0
error-project-scale-not-allowed = scale 1:{ $scale } is not allowed for { $kind } maps; use { $allowed }
error-project-scale-not-allowed-enlarge = scale 1:{ $scale } is not allowed for { $kind } maps; use { $allowed }. Draw at 1:4000, freemapper-print enlarges to 1:3000 when printing
word-kind-schoolyard = schoolyard
word-kind-sprint = sprint
word-kind-forest = forest
error-project-toml = invalid project file: { $message }
error-project-unknown-adapter = the project file names the adapter "{ $adapter }", which this freemapper does not have; a newer freemapper may bring it (`freemapper update`), or it comes from your adapter folder (`freemapper adapters list` shows it) or, with --adapters-from-project on the command line, from the folder `adapters` of the project (only for a project you trust)
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
error-project-outside-coverage = the map area lies outside the supported states ({ $states }); freemapper has data sources for those only so far

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
word-part-point-cloud = the laser point clouds for the vegetation candidates

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
error-fetch-not-allowed = { $url }: freemapper does not send requests to this address: { $reason }
error-fetch-host-not-named-allow = { $target }: freemapper does not send requests to this address: the adapter of { $name } does not name the host { $host }{ $from }. If the service has moved there, allow the host with `freemapper adapters allow-host { $adapter } { $host }` and report it at https://github.com/HenryJobst/freemapper/issues
error-fetch-host-not-named-manifest = { $target }: freemapper does not send requests to this address: the adapter { $adapter } does not name the host { $host }{ $from }. If the service has moved there, add the host to `[net] hosts` of the adapter's manifest
error-fetch-host-redirected-from = {" "}({ $url } redirects there)
error-reason-not-a-host-name = not a host name
error-reason-not-ascii = a name outside ASCII must be written in its xn-- form
error-reason-not-a-dns-name = not a DNS name
error-reason-no-domain = a name of this computer or without a domain
error-reason-local-name = a name of the local network
error-reason-ip-address = an IP address, not a DNS name
error-reason-unexpected-character = unexpected character
error-reason-not-https = not https
error-reason-user-info = user info
error-reason-port = a port other than 443
error-reason-host-not-declared = the host is not one of the declared hosts
error-reason-local-address = an address of this computer or of the local network, not of a server on the internet
error-reason-too-many-redirects = more than { $count } redirects
error-reason-redirect-nowhere = a redirect (HTTP { $status }) to nowhere
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
help-arg-scale = Scale denominator of a new project: schoolyard 1000 (default); sprint 4000 (default) or 5000; forest 15000 (default), 10000 or 7500. Other values are refused, because freemapper-print refuses them
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
help-derive = Compute the templates (orthophoto, hillshade, slope, local relief, nDSM), contours and landforms from the fetched data
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
help-adapters = Show the region adapters: the states and countries freemapper has data sources for
help-adapters-list = List the adapters with where each comes from (built in, your folder, a project), its revision, the day it was last checked and the hosts it asks
help-adapters-list-json = Print the list as JSON
help-adapters-list-project = Also read the adapters in the folder `adapters` of this project folder, as --adapters-from-project would
help-adapters-check = Check the folder of an adapter before it is used or passed on: whether it would load, what its author should know, and with --live whether its services answer. Exit code 0: it passes; 1: it has findings; 2: the check could not run
help-adapters-check-dir = The folder of the adapter; its name is the id of the adapter
help-adapters-check-live = Also ask the services of the adapter for the box --bbox names and say what each delivers. The requests stay on the hosts the manifest declares
help-adapters-check-bbox = The box of the live check in degrees, as minLon,minLat,maxLon,maxLat. It lies inside the boundary of the adapter; 150 m by 150 m is enough
help-adapters-check-json = Print the result as JSON
help-adapters-allow-host = Allow a built-in adapter one more host: for a state whose service moved to a server its adapter does not name yet. The message about the refused address says which host. The host is checked like a host of a manifest, and freemapper still connects to servers on the internet only
help-adapters-allow-host-adapter = The id of the built-in adapter, as `adapters list` shows it
help-adapters-allow-host-host = The host to allow: a DNS name, no address, no port
help-adapters-disallow-host = Take back a host that allow-host allowed
help-adapters-disallow-host-adapter = The id of the built-in adapter
help-adapters-disallow-host-host = The host to take back
help-arg-adapters-from-project = Also load the adapters in the folder `adapters` of the project folder. freemapper names each of them with the hosts it asks. Use it for a project you trust: an adapter makes freemapper ask the servers it names

## The summary of a run, for the MCP server and the area page

cli-internal-error = internal error: { $message }
cli-pdf-sources-invalid = pdf_sources must be auto, full or credit-only, not { $value }
cli-estimate-disk-unreadable = cannot read the free disk space of { $path } ({ $message })
cli-summary-wrote-map = Wrote the map { $path }
cli-summary-kept-map = Kept the existing map { $path } (it may contain the mapper's work)
cli-summary-map-lacks-templates = The map does not reference the image templates yet. Start the pipeline with force to replace it and keep the old one as .omap.bak.
cli-summary-map-lacks-buildings = The map has no ALKIS buildings yet. Start the pipeline with force to add them; it replaces the map and keeps the old one as .omap.bak.
cli-summary-map-lacks-canopies = The map lacks the ALKIS canopies and carports (522) that are now map objects. Start the pipeline with force to add them; it replaces the map and keeps the old one as .omap.bak.
cli-summary-wrote-ocad = Wrote the OCAD map { $path } ({ $count } objects)
cli-summary-kept-ocad = Kept the existing OCAD map { $path } (it may contain the mapper's work; force replaces it)
cli-summary-ocad-left-out = Left out of the OCAD map: { $skipped } objects and { $dropped } degenerate parts
cli-summary-ocad-symbols = In OCAD, choose Symbol > Load Colors and Symbols From… and pick the official symbol set{ $kind ->
        [forest] {" "}ISOM 2017-2
        [schoolyard] , the IOF school set 2019
       *[other] {" "}ISSprOM 2019
    }; it replaces the placeholder symbols by number.
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
mcp-error-unknown-kind = unknown kind { $kind }; use "schoolyard", "sprint" or "forest"
mcp-error-reading = reading { $path }: { $error }
mcp-error-one-area-source = give exactly one of osm_id (from find_place) or geojson
mcp-error-page-open = the area selection page for this project is still open; ask the user to save the area and press Done on the page, then start the pipeline
mcp-error-no-job = no job { $job }; start_pipeline and permits_for_area return the job ids of this session
mcp-error-job-running = { $job } is still running for { $path }; one pipeline runs at a time because projects share the download cache. Start this one when project_status reports finished or failed
mcp-error-page-start = cannot start the page: { $error }
mcp-created-project = Created project { $path } for { $area } ({ $kind }, 1:{ $scale }). Next: estimate_download, then start_pipeline.
mcp-created-project-given-area = Created project { $path } for the given area ({ $kind }, 1:{ $scale }). Next: estimate_download, then start_pipeline.
mcp-page-already-open = The page for this folder is already open; it keeps the settings it was opened with.
mcp-page-next = Ask the user to choose the area on the page, press Save and then Done.
mcp-preview-of = Preview of { $layer }
mcp-permits-next = The permit notes are being made. Poll project_status with this job_id; when it is finished, permits_md holds the text of permits.md. Pass it on as a hint, not legal advice.
mcp-tool-check-update = Whether a newer freemapper release exists: its version, release date, release notes and download size, or that this one is up to date. Installs nothing.
mcp-tool-create-project = Create a project folder for a map area. Give osm_id (from find_place) or a GeoJSON file. kind is schoolyard (1:1000, default), sprint (1:4000) or forest (1:15000).
mcp-param-create-project-dir = New project folder (absolute path or starting with ~).
mcp-param-create-project-geojson = Instead of osm_id: a GeoJSON file with the map area polygon (EPSG:4326).
mcp-param-create-project-kind = "schoolyard" (default, 1:1000), "sprint" (1:4000) or "forest" (1:15000).
mcp-param-create-project-osm-id = OSM object of the map area from find_place, e.g. "way/992423119".
mcp-param-create-project-part = With osm_id: which part of an area of several separate parts to use (1 is the largest).
mcp-param-create-project-scale = Scale denominator: schoolyard 1000 (default); sprint 4000 (default) or 5000; forest 15000 (default), 10000 or 7500. Other values are refused; for 1:3000 draw at 4000, freemapper-print enlarges when printing.
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
mcp-param-open-area-selector-kind = Map kind of a new project: "schoolyard" (default, 1:1000), "sprint" (1:4000) or "forest" (1:15000).
mcp-param-open-area-selector-ocad = The run panel starts with the OCAD starter project ticked. Default false.
mcp-param-open-area-selector-offline = The run panel starts with "cache only, no network" ticked. Default false.
mcp-param-open-area-selector-open-browser = Also open the page in the default browser of this computer. Default true.
mcp-param-open-area-selector-pdf = The run panel starts with the printable PDF ticked. Default false.
mcp-param-open-area-selector-pdf-candidates = The run panel starts with the training PDF (every candidate) ticked. Default false.
mcp-param-open-area-selector-pdf-scale-bar = The run panel starts with the scale bar on the PDFs ticked. Default false.
mcp-param-open-area-selector-pdf-sources = Source lines on the PDFs the panel starts with: "auto" (default), "full" or "credit-only", as for start_pipeline.
mcp-param-open-area-selector-permits = The run panel starts with the permit notes ticked. Default false.
mcp-param-open-area-selector-scale = Scale denominator of a new project: schoolyard 1000 (default); sprint 4000 (default) or 5000; forest 15000 (default), 10000 or 7500. Other values are refused.
mcp-param-open-area-selector-title = Map title of a new project.
mcp-tool-preview-layer = A PNG preview (at most 800 px) of a layer: map (the image of the map objects that the last run wrote, without the candidates), or a raster template: orthophoto, hillshade, slope, relief, ndsm, or k5 in Berlin.
mcp-param-preview-layer-dir = Project folder (absolute path or starting with ~).
mcp-param-preview-layer-layer = "map", "orthophoto", "hillshade", "slope", "relief", "ndsm" or "k5" (Berlin only).
mcp-tool-project-status = State of a job of start_pipeline or permits_for_area: running, finished or failed, the last progress lines, warnings, the files written, and the result or the error. A finished permits_for_area job also carries the text of permits.md as permits_md.
mcp-param-project-status-job-id = Job id from start_pipeline or permits_for_area, e.g. "job-1".
mcp-tool-permits-for-area = Make the permit notes of a project: the administrative units of the map area, the protected areas in it and who runs the land there, with the authorities to ask; a hint, not legal advice. Writes permits.md and permits.json into the folder. Runs in the background and returns a job_id; poll project_status, whose answer carries the text of permits.md as permits_md when the job is finished. One job runs at a time.
mcp-param-permits-for-area-dir = Project folder (absolute path or starting with ~).
mcp-param-permits-for-area-offline = Read only the data stored in the project by an earlier run, without network access. Default false.
mcp-tool-start-pipeline = Download the data, compute the templates and write the starter project (fetch, derive, export). Runs in the background and returns a job_id; poll project_status. One pipeline runs at a time. It keeps an existing map unless force is true.
mcp-param-start-pipeline-dir = Project folder (absolute path or starting with ~).
mcp-param-start-pipeline-force = Replace an existing map file and keep the old one as .omap.bak. Default false.
mcp-param-start-pipeline-ocad = Also write an OCAD starting point into the folder ocad/ (OCAD 12 format, for users who edit in OCAD). Default false.
mcp-param-start-pipeline-offline = Use the download cache only, without network access. Default false.
mcp-param-start-pipeline-pdf = Also write <name>.pdf, the map ready to print (the map objects only, no candidates). Default false.
mcp-param-start-pipeline-pdf-candidates = Also write <name>-candidates.pdf, the training map (the map plus every candidate drawn as a map object; check them in the terrain). Default false.
mcp-param-start-pipeline-pdf-scale-bar = Draw the scale bar on the PDFs. Default false (German maps usually have none; the scale number stays in the title block).
mcp-param-start-pipeline-pdf-sources = Source lines on the PDFs: "auto" (default; the full list, shortened only when it takes more than 5 lines), "full" (never shortened) or "credit-only" (only the freemapper credit line; ODbL and dl-de/by-2-0 then require naming the sources when the map is passed on). ATTRIBUTION.txt always lists every source.
mcp-param-start-pipeline-permits = Also write the permit notes (permits.md and permits.json) after the map: administrative units, protected areas and who runs the land. A failure there is a warning, not a failed job. Default false.
mcp-instructions = freemapper makes OpenOrienteering Mapper starter projects for orienteering maps (schoolyard 1:1000, sprint 1:4000, forest 1:15000) from official geodata of { $states }, and from OSM. Workflow: find_place (search the school or area by name; pick a result with has_area) → create_project (a new folder, absolute path, with the osm_id) → estimate_download (tell the user how much will be downloaded) → start_pipeline (returns a job id; downloading can take minutes) → project_status (poll every few seconds until the state is finished or failed) → list_layers and preview_layer (show the user the templates). The user then opens the map file in OpenOrienteering Mapper. When no place fits or the user wants to draw the area, open_area_selector opens a page in the browser where the user draws or picks the area and saves it (this creates the project); continue with estimate_download once the user says it is saved. Paths must be absolute or start with ~. One pipeline runs at a time. permits_for_area makes the permit notes of a project (administrative units, protected areas, who runs the land; a hint, not legal advice) as a job like start_pipeline. check_update tells whether a newer freemapper exists; install_update installs it, only with confirm: true after the person agreed, and the assistant then restarts the freemapper server.

cli-contour-interval-saved = Contour interval saved in freemapper.toml: { $interval } m; derive traces the contours at it
cli-contour-interval-auto = Contour interval in freemapper.toml: automatic (the map kind's; a forest map gets 2.5 m in flat land and 10 m in steep land)
help-arg-contour-interval = Contour interval in metres, e.g. `2.5`, or `auto` for the map kind's (a forest map: 5 m, 2.5 m where it is flat, 10 m where it is steep). Saved in freemapper.toml; derive traces the contours at it
cli-content-saved = Content setting saved in freemapper.toml: { $summary }
cli-content-invalid = Cannot read the content setting: { $detail }
help-arg-content = What goes straight onto the map: `safe` leaves every object where the build puts it, `balanced` (default) puts the objects of the cadastre and the registers on the map, `full` puts everything on the map. The choice is saved in freemapper.toml
help-arg-content-set = Send a group of objects to `map`, `candidates` or `off`, as `GROUP=PLACE`, e.g. `walls=map`; may be repeated. Groups: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets. Saved in freemapper.toml
mcp-param-create-project-content = What goes straight onto the map: "safe", "balanced" (default) or "full". Saved in the project. balanced puts the objects of the cadastre and the registers on the map, full puts everything on it.
mcp-param-create-project-content-set = Groups sent somewhere else than the profile says, as "group=place" with place map, candidates or off, e.g. "walls=map". Groups: walls, fences, hedges, slopes, stairs, trees, water-features, monuments, furniture, passages, contours, landforms, vegetation, out-of-bounds, paths, streets.
mcp-param-create-project-contour-interval = Contour interval in metres, e.g. 2.5; 0 goes back to the automatic choice (the map kind's; a forest map gets 2.5 m in flat land and 10 m in steep land). Saved in the project; the next derive traces the contours at it.
mcp-param-start-pipeline-content = Change what goes straight onto the map before the run: "safe", "balanced" or "full". Saved in the project. Default: the project's setting, else balanced.
mcp-param-start-pipeline-content-set = Groups sent somewhere else than the profile says, as "group=place" with place map, candidates or off, e.g. "fences=candidates". Saved in the project.
mcp-param-start-pipeline-contour-interval = Contour interval in metres, e.g. 2.5; 0 goes back to the automatic choice (the map kind's; a forest map gets 2.5 m in flat land and 10 m in steep land). Saved in the project; the next derive traces the contours at it.
mcp-error-content = Cannot read the content setting: { $detail }

## Permit notes (`freemapper permits`, `run --permits`, ADR 0117). The ids `permits-hint-*`
## are the keys the engine stores in permits.json; `src/permits.rs` checks that every
## `permits-*` id is used. The quote "Nicht für Planungszwecke geeignet." is the BfN's own
## wording and stays German in every language.

help-permits = Write the permit notes of a project (permits.md and permits.json): the administrative units, protected areas and operators the map area touches, and whom that usually means asking. It asks the BfN and the BKG, for a map area in Berlin also the Geoportal Berlin (gdi.berlin.de), in Hamburg the Geoportal Hamburg (geodienste.hamburg.de), in Brandenburg the Landesbetrieb Forst (www.brandenburg-forst.de), in North Rhine-Westphalia Wald und Holz NRW (www.opengeodata.nrw.de, about 17 MB), each through the hosts of the state's adapter, and reads the OSM extract already cached; --offline uses what was fetched before
help-permits-dir = Project folder
help-permits-offline = Use the data fetched before only, without network access
help-run-permits = Also write the permit notes (permits.md and permits.json) after the map; asks the BfN and the BKG, in Berlin and Hamburg also their Geoportal, in Brandenburg and North Rhine-Westphalia the forest administration. A failure there is a warning and the map stays
cli-wrote-permits = Wrote the permit notes { $path }
cli-summary-wrote-permits = Wrote the permit notes { $path }
cli-permits-failed = the permit notes could not be written: { $error }
cli-permits-source-left-out = the permit notes leave out { $source }: { $reason }
cli-permits-not-read = it was not read
error-permits-stored = { $path }: { $message }; fetch the permit notes again online
error-permits-nothing-read = no source of the permit notes could be read ({ $missing }); fetch them online first
error-permits-missing-not-read = { $source }: not read

permits-title = Permit notes
permits-disclaimer = A hint, not legal advice. The competent authorities decide. For the data date and accuracy see the table under "Sources and data date"; areas can be missing or out of date (BfN: merged once a year, "Nicht für Planungszwecke geeignet.", that is, not suitable for planning).
permits-bfn-merge = The BfN merges the data of the states once a year and marks them as not suitable for planning. The authorities of the state decide.
permits-created = Made on { $date } for the map area of a { $kind ->
        [schoolyard] schoolyard map
        [forest] forest map
       *[other] sprint map
    } ({ $area } ha).
permits-section-admin = Administrative units
permits-section-protected = Protected areas (BfN)
permits-section-osm-protected = More protected areas from OpenStreetMap
permits-section-operators = Operators and owners according to OpenStreetMap
permits-section-hints = Hints
permits-section-sources = Sources and data date
permits-none-admin = VG250 names no administrative unit for the map area.
permits-none-protected = According to the BfN, no protected area lies in the map area.
permits-none-osm-protected = OpenStreetMap names no protected area in the map area that the BfN does not list already.
permits-none-operators = OpenStreetMap names no operator and no owner for the map area.
permits-none-hints = No hints.
permits-section-not-read = The data of { $source } were not read, so this section is empty, although there may be entries.
permits-source = { $source ->
        [vg250] VG250 (BKG)
        [bfn] BfN
       *[osm] OpenStreetMap
    }
permits-source-what = { $source ->
        [vg250] Administrative areas 1:250 000 (VG250) of the BKG
        [bfn] Protected areas in Germany of the BfN
       *[osm] OpenStreetMap, from the extract freemapper keeps in its cache
    }
permits-level = { $level ->
        [land] State
        [kreis] District (Kreis)
        [verwaltungsgemeinschaft] Amt or Verwaltungsgemeinschaft
       *[gemeinde] Municipality (Gemeinde)
    }
permits-admin-line = { $level }: { $name }, { $share }
permits-share = { $percent }%
permits-share-below-one = < 1%
permits-at-edge = Touched at the edge (less than 0.5% of the map area):
permits-at-edge-generalised = VG250 is generalised to the scale 1:250 000. A unit that only touches the edge can be an effect of that.
permits-kind = { $kind ->
        [nationalpark] National park
        [nationales-naturmonument] National natural monument
        [naturschutzgebiet] Nature reserve (Naturschutzgebiet)
        [ffh] FFH area (Natura 2000)
        [vogelschutzgebiet] Bird protection area (Natura 2000)
        [biosphaerenreservat] Biosphere reserve
        [biosphaerenreservat-zone] Zone of a biosphere reserve
        [landschaftsschutzgebiet] Landscape protection area (Landschaftsschutzgebiet)
       *[naturpark] Nature park
    }
permits-protected-line = { $kind } "{ $name }", { $share }
permits-protected-date = Designated on { $date }
permits-protected-title = Legal basis: { $title }
permits-protected-link = Legal text: { $url }
permits-protected-site-link = Site page (Natura 2000): { $url }
permits-protected-year = Data year at the BfN: { $year }
permits-osm-line = { $kind }: { $name }, { $share } (OSM { $id })
permits-osm-protect-class = protect class { $class }
permits-quoted = "{ $text }"
permits-osm-unnamed = without a name
permits-osm-operator = Operator according to OSM: { $operator }
permits-osm-enclosing-gap = OpenStreetMap areas that enclose the whole map area without a corner inside it can be missing here, and so can protected areas mapped in OSM as a relation with `type=boundary`. The parts from the BfN and VG250 are not affected.
permits-operator-line = { $name } ({ $roles ->
        [operator] operator
        [owner] owner
       *[both] operator and owner
    }): { $objects }
permits-column-source = Source
permits-column-state = State
permits-column-credit = Credit and licence
permits-state-read = read, fetched on { $date }
permits-state-read-osm = read, OSM extract of { $date }
permits-state-unreachable = not reachable ({ $reason })
permits-state-not-read = not read
permits-no-credit = not used
permits-hint-city-state = The map area lies in Berlin or Hamburg. There the Bezirksamt (district office) or an authority of the state is usually in charge. These notes do not determine the Bezirk yet.
permits-hint-nature-authority = { $kreisfrei ->
        [yes] Protected areas concern the lower nature conservation authority (untere Naturschutzbehörde). Here that is the city of { $kreis } itself.
        [no] Protected areas concern the lower nature conservation authority (untere Naturschutzbehörde), usually the { $kreis }.
       *[unknown] Protected areas concern the competent lower nature conservation authority (untere Naturschutzbehörde), usually the Landkreis or the kreisfreie Stadt.
    }
permits-hint-national-park = A national park has rules of its own. Ask the national park administration early.
permits-hint-nsg-paths = In a nature reserve you usually have to keep to the paths. For a run off the paths, ask the nature conservation authority.
permits-hint-natura2000 = An FFH area or a bird protection area (Natura 2000) touches the map area. For an event the nature conservation authority may ask for an assessment of its effects.
permits-hint-order-office = { $amt ->
        [none] For streets, squares and other public land the Ordnungsamt (public order office) of the Gemeinde { $gemeinde } is usually in charge.
       *[other] For streets, squares and other public land the Ordnungsamt (public order office) is usually in charge: { $amt }, for the Gemeinde { $gemeinde }.
    }
permits-hint-park-operator = Operator of a park or garden in the map area according to OpenStreetMap: { $operator }. Ask them before you use it.
permits-hint-school-operator = Operator of the school or institution according to OpenStreetMap: { $operator }. For its grounds, ask the head of the school or this body.
permits-section-districts = Bezirke (districts, { $source })
permits-section-green-spaces = Green spaces ({ $source })
permits-section-owners = Land of the state and of the Bund ({ $source })
permits-section-monuments = Monuments ({ $source })
permits-section-protected-objects = Natural monuments and other protected objects ({ $source })
permits-none-districts = According to { $source }, the map area lies in no Bezirk.
permits-none-green-spaces = According to { $source }, no public green space lies in the map area.
permits-none-owners = According to { $source }, no parcel of the state or of the Bund lies in the map area.
permits-none-monuments = According to { $source }, no monument lies in the map area.
permits-none-protected-objects = According to { $source }, no natural monument, no protected landscape feature and no area with special rules lies in the map area.
permits-share-line = { $name }: { $share }
permits-count-line = { $name }: { $count }
permits-green-space-district = Bezirk { $district }
permits-green-space-dedication = dedication: { $dedication }
permits-playgrounds-none = No public playground lies in the map area.
permits-playgrounds-one = One public playground lies in the map area. It is not listed on its own.
permits-playgrounds-many = { $count } public playgrounds lie in the map area. They are not listed one by one.
permits-owner-other = Other public owners
permits-owners-rest = For the rest of the area the data name no owner.
permits-monuments-total = Monuments wholly or partly in the map area: { $count }. The links lead to their entries in the list of monuments.
permits-monuments-links-left = { $count } more entries are only counted here, without a link.
permits-object-kind = { $kind ->
        [natural-monument] Natural monument (Naturdenkmal)
        [protected-landscape-part] Protected landscape feature (geschützter Landschaftsbestandteil)
       *[special-rules] Area with special rules
    }
permits-object-count = { $count } objects
permits-object-citations = Legal bases:
permits-source-note = { $source }: "{ $note }"
permits-hint-district-office = { $offices ->
        [hamburg] For streets, squares and green spaces in the Bezirk { $district }, the Bezirksamt { $district } is usually in charge, there the Fachamt Management des öffentlichen Raumes (office for public space).
       *[berlin] For streets, squares and green spaces in the Bezirk { $district }, the Ordnungsamt (public order office) and the Straßen- und Grünflächenamt (roads and parks office) of the Bezirksamt { $district } are usually in charge.
    }
permits-hint-monument-authority = { $offices ->
        [hamburg] For the monuments in the map area, the Denkmalschutzamt Hamburg (monument office) is in charge.
       *[berlin] For the monuments in the Bezirk { $district }, the lower monument authority (untere Denkmalschutzbehörde) of the Bezirksamt { $district } is in charge.
    }
permits-hint-nature-object-authority = { $offices ->
        [hamburg] For natural monuments and protected landscape features in the Bezirk { $district }, the Bezirksamt { $district } is usually in charge.
       *[berlin] For natural monuments and protected landscape features in the Bezirk { $district }, the lower nature conservation authority (untere Naturschutzbehörde) of the Bezirksamt { $district } is in charge.
    }
permits-hint-owner-share = { $unnamed ->
        [yes] According to the open data of the state, { $share } of the map area belongs to other public owners.
       *[no] According to the open data of the state, { $share } of the map area belongs to the owner "{ $owner }".
    }
permits-hint-owners-unknown = { $rest ->
        [yes] The owners of the rest of the area cannot be found from open data. With a legitimate interest (an organiser, for example) you can ask the land registry (Grundbuchamt) for an extract; for public land the competent Ordnungsamt or parks department helps.
       *[no] The owners of these areas cannot be found from open data. With a legitimate interest (an organiser, for example) you can ask the land registry (Grundbuchamt) for an extract; for public land the competent Ordnungsamt or parks department helps.
    }
permits-section-forest-units = Forest administration ({ $source })
permits-section-state-forest = State forest ({ $source })
permits-none-forest-units = According to { $source }, the map area lies in the area of no forest office.
permits-none-state-forest = According to { $source }, no state forest lies in the map area.
permits-forest-units-reach = A forest office is in charge of the forest in its area. Its area can take in towns and fields as well.
permits-state-forest-office = in charge: { $office }
permits-hint-forest-office = { $units ->
        [none] Forest office in charge of the forest in the map area: { $office } ({ $administration }). Ask there early when you plan an event in the forest.
       *[other] Forest office in charge of the forest in the map area: { $office } ({ $administration }), { $units }. Ask there early when you plan an event in the forest.
    }
permits-hint-forest-owner = { $office ->
        [known] An event in the forest needs the consent of the forest owners. The forest office can usually say who owns the forest. In several states an organised event in the forest must also be reported to the forest authority or allowed by it.
       *[unknown] An event in the forest needs the consent of the forest owners. In several states an organised event in the forest must also be reported to the forest authority or allowed by it. These notes do not name the forest office in charge here; the forest administration of the state or the lower forest authority can tell you.
    }
permits-section-water-protection = Water protection areas ({ $source })
permits-section-forest-functions = Forest functions and protected forest ({ $source })
permits-none-water-protection = According to { $source }, no water protection area lies in the map area.
permits-none-forest-functions = According to { $source }, no recreation forest, no protected forest, no natural forest and no closed forest area lies in the map area.
permits-water-zone = { $zone }
permits-forest-function-kind = { $kind ->
        [recreation] Recreation forest (Erholungswald)
        [protected] Protected forest (Bannwald or Schutzwald)
        [natural] Natural forest or strict reserve (Naturwald, Totalreservat)
       *[closed] Closed forest area
    }
permits-hint-water-protection = { $inner ->
        [yes] The { $zone } of the water protection area "{ $name }" lies in the map area. In zones I and II events are usually ruled out. Ask the lower water authority (untere Wasserbehörde).
        [no] The { $zone } of the water protection area "{ $name }" lies in the map area. Events there are usually possible; the lower water authority (untere Wasserbehörde) can say what the ordinance allows.
       *[unknown] The water protection area "{ $name }" lies in the map area. The data do not say which zone it is. In zones I and II events are usually ruled out; ask the lower water authority (untere Wasserbehörde).
    }
permits-hint-forest-function = { $kind ->
        [recreation] Recreation forest (Erholungswald) lies in the map area{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. For events there, ask the forest authority.
        [protected] Protected forest (Bannwald or Schutzwald) lies in the map area{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Ask the forest authority before you plan an event there.
        [natural] Natural forest or a strict reserve lies in the map area{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Leaving the paths is often forbidden there; place no controls in it without asking the forest authority.
       *[closed] A forest area that must not be entered lies in the map area{ $names ->
            [none] {""}
           *[other] {" "}({ $names })
        }. Keep it out of the course.
    }
permits-hint-state-forest = { $office ->
        [none] According to the open data of the state, { $share } of the map area is state forest of the owner "{ $owner }".
       *[other] According to the open data of the state, { $share } of the map area is state forest of the owner "{ $owner }". In charge: { $office }.
    }
