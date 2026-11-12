# Browser check of the run panel (by hand, not in CI)

Needs `playwright-cli` (the `playwright` skill). It sends no personal data. The page loads a few map
tiles through the local server.

1. Start the page on a scratch folder with offline defaults and a private cache:
   `freemapper select <scratch>/proj --no-browser --lang de --offline --pdf --pdf-sources credit-only --cache-dir <scratch>/cache`
   It prints `http://127.0.0.1:<port>/<token>/`. `--lang de` gives the German words below; without it the page
   follows the environment, then the browser.
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
7. Two windows: `tab-new <url>`, change the area in the new tab (draw a rectangle) and click "Speichern", then go
   back to the first tab (`tab-select 0`), change its area and click "Speichern": the banner says that
   freemapper.toml was changed in another window, nothing is saved, and a second "Speichern" saves.
8. Dark mode: `set-color-scheme dark` and `screenshot`: the panel turns dark, the map stays light.
9. Content profile: open "Was auf die Karte kommt" (eingeklappt), choose "Voll" and set "Mauern" to "Aus", click
   "Speichern": `freemapper.toml` gets `[content]` with `profile = "full"` and `walls = "off"`. Reload: "Voll" and
   "Aus" are still chosen, the other groups say "Wie im Profil". Change only the profile to "Sicher" and click
   "Karte erzeugen" with a mocked `api/run`: the page saves first (`api/save` carries `content`) and the
   exception stays.
10. Layer list: the control at the top right of the map lists "OpenStreetMap" and ten aerial photos, one per
    agency ("Luftbild Brandenburg/Berlin (LGB)" … "Luftbild Rheinland-Pfalz (LVermGeo RP)"); with `--lang en` they
    read "Aerial photo …". Choose each of the ten once: its tiles load (`.leaflet-tile-pane img` with an `https`
    source, `naturalWidth > 0`), the credit at the bottom right names the agency, and `console` stays empty. A
    blocked or misspelled service shows as tiles that never load. Repeat one layer with `set-color-scheme dark`:
    the control turns dark, the map stays light. Then answer `api/adapters` with status 500 (`route`) and reload:
    the page starts, shows the project folder, lists OpenStreetMap only and says why in its message line.
