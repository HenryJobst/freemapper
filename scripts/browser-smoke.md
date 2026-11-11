# Browser check of the run panel (by hand, not in CI)

Needs `playwright-cli` (the `playwright` skill). It sends no personal data. The page loads a few map
tiles through the local server.

1. Start the page on a scratch folder with offline defaults and a private cache:
   `freemapper select <scratch>/proj --no-browser --offline --pdf --pdf-sources credit-only --cache-dir <scratch>/cache`
   It prints `http://127.0.0.1:<port>/<token>/`.
2. Save a small area (a script, with the page's origin and JSON):
   `curl -s -X POST -H "Origin: http://127.0.0.1:<port>" -H "Content-Type: application/json" -d '{"area":{"type":"Polygon","coordinates":[[[13.880,52.580],[13.885,52.580],[13.885,52.583],[13.880,52.583],[13.880,52.580]]]},"kind":"schoolyard","scale":1000,"title":"Scratch"}' <url>api/save`
3. `playwright-cli open <url>`, then `snapshot`: the block "Karte erzeugen" shows the defaults (PDF and offline
   ticked, source lines "nur freemapper" with the licence hint).
4. Click "Karte erzeugen". With an empty cache and `--offline` the run fails at once: the result region has the
   focus, shows "Lauf fehlgeschlagen" with the engine's message, and the button reads "Nochmal erzeugen".
5. `touch <scratch>/proj/proj.omap`, reload, tick "Vorhandene Karte ersetzen", click "Karte erzeugen": the dialog
   opens with "Abbrechen" focused; Escape closes it and no run starts.
6. Progress and a finished run need a real fetch. For a picture, mock `api/run` (202) and `api/status`
   (state `running`, a `phase`, some `lines`) with `playwright-cli run-code` and `page.route`.
