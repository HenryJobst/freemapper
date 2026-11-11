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
