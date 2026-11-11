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
