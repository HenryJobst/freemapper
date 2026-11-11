# freemapper for AI agents

This file tells an AI agent how to install freemapper and how to use it for a person who wants an orienteering
map. It ships in every release archive next to the program. People read [README.en.md](README.en.md) (English) or
[README.md](README.md) (German); the [user manual](docs/manual/en.md) has the details.

freemapper makes a starter project for an orienteering map from official geodata of a German state and from
OpenStreetMap: a georeferenced OpenOrienteering Mapper map (`.omap`) with buildings, paved areas, paths and other
reliable objects already drawn, templates (orthophoto, hillshade, slope, nDSM), a candidates map to check, and on
request an OCAD starting point and printable PDFs. Map kinds: `schoolyard` (1:1000) and `sprint` (ISSprOM 2019,
1:4000 or 1:5000).

Supported states: Brandenburg, Berlin, Saxony, Saxony-Anhalt, Hamburg, Hesse, Schleswig-Holstein,
Mecklenburg-Vorpommern, North Rhine-Westphalia, Lower Saxony and Rhineland-Palatinate. Bremen, Thuringia, Bavaria,
Baden-Württemberg and Saarland have no data sources yet; an area there fails with a message that says so. Tell the
user before you start if the place lies outside these states.

## Install

freemapper is one executable without an installer and without dependencies. Do not build it from source unless
the user asks; the release archives are ready to run.

1. Find the newest version: `https://api.github.com/repos/HenryJobst/freemapper/releases/latest` returns JSON with
   `tag_name`, for example `v0.12.0`. The release page is
   <https://github.com/HenryJobst/freemapper/releases/latest>.
2. Pick the archive for the system. The name is `freemapper-<tag>-<platform>` plus the extension:

   | System (`uname -s` / `uname -m`) | Platform | Archive |
   |---|---|---|
   | macOS, `arm64` (Apple chip) | `macos-arm64` | `.tar.gz` |
   | macOS, `x86_64` (Intel) | `macos-x86_64` | `.tar.gz` |
   | Linux, `x86_64` | `linux-x86_64` | `.tar.gz` |
   | Linux, `aarch64` | `linux-arm64` | `.tar.gz` |
   | Windows, x64 | `windows-x86_64` | `.zip` |

   Each archive has a `<archive>.sha256` next to it. The Linux builds are static (musl) and run on any
   distribution. The release also holds `freemapper-<tag>-<platform>.gz` and `freemapper-update.json` with its
   `.minisig`; they are for `freemapper update`, not for the first install.
3. Download, check and unpack. macOS and Linux, for example in `~/Applications` or `~/opt`:

   ```sh
   tag=v0.12.0; platform=macos-arm64
   base="https://github.com/HenryJobst/freemapper/releases/download/$tag"
   curl -fLO "$base/freemapper-$tag-$platform.tar.gz"
   curl -fLO "$base/freemapper-$tag-$platform.tar.gz.sha256"
   shasum -a 256 -c "freemapper-$tag-$platform.tar.gz.sha256"   # Linux: sha256sum -c
   tar -xzf "freemapper-$tag-$platform.tar.gz"
   ```

   Windows (PowerShell):

   ```powershell
   $tag = "v0.12.0"; $name = "freemapper-$tag-windows-x86_64.zip"
   $base = "https://github.com/HenryJobst/freemapper/releases/download/$tag"
   Invoke-WebRequest "$base/$name" -OutFile $name
   Invoke-WebRequest "$base/$name.sha256" -OutFile "$name.sha256"
   (Get-FileHash $name -Algorithm SHA256).Hash.ToLower()   # compare with the first word of the .sha256 file
   Expand-Archive $name -DestinationPath .
   ```

4. The archive unpacks to a folder `freemapper-<tag>-<platform>/` with `freemapper` (Windows: `freemapper.exe`),
   the READMEs, `LICENSE`, this file and `docs/manual/`. The program is not signed:
   - macOS refuses to start it until the quarantine flag is gone:
     `xattr -d com.apple.quarantine freemapper-<tag>-<platform>/freemapper`. Tell the user you did this and why.
   - Linux: `chmod +x freemapper` if the executable bit got lost.
   - Windows may show "Windows protected your PC" on the first start. `Unblock-File $name` before
     `Expand-Archive` avoids it; otherwise the user confirms with *More info* > *Run anyway*.
5. Check: `<folder>/freemapper --version` prints `freemapper <version>`.

Remember the absolute path of the executable; the MCP registration needs it.

To update, run `<path>/freemapper update --yes` after the person agreed. It replaces the program at the same
path, so the MCP registration stays valid; restart the MCP server afterwards (see "Use it through MCP"). It
installs only a release whose signature checks out, keeps the old program as `freemapper.previous`
(`freemapper.previous.exe` on Windows), and `freemapper update --rollback` brings it back. Versions 0.11.0 and
older have no `update` command: unpack the newest archive as above, once, and point the registration at the new
path if it changed. A program built from source answers `update` with how to update it with git and cargo
instead. Projects and the download cache stay valid either way.

Do not add the user's e-mail address, name or other personal data to any request, file name, project title or
setting. freemapper itself sends only a fixed `User-Agent` that names the program and its repository.

## Use it from the command line (preferred)

If you can run commands on the person's computer (Claude Code, Codex and similar), use the command line. One
`freemapper run` call blocks until the map is written; through MCP you start a job and poll it, and every poll is a
turn that costs tokens. Use MCP only when you cannot run commands (see the next section).

`freemapper --help` lists the commands, `freemapper <command> --help` their switches. Pass `--lang en` when you
parse the output; this file quotes the English texts.

```sh
freemapper init <dir> --place "Clara-Zetkin-Schule Strausberg" --kind schoolyard --title "Clara-Zetkin-Schule"
freemapper run <dir> --pdf
```

| Command | Purpose |
|---|---|
| `init <dir>` with `--geojson <file>`, `--osm way/<id>` (plus `--part <n>`) or `--place <name>` | Creates the project. `--kind schoolyard\|sprint`, `--scale`, `--title`. The GeoJSON holds one polygon in EPSG:4326. |
| `select <dir>` | Opens the area page in the browser for the user. `--no-browser` only prints the address. A person must draw and save; you cannot do it for them. |
| `run <dir>` | `fetch`, `derive` and `export` in one go. |
| `fetch <dir>` | Downloads into the shared cache. `--offline` uses the cache only. |
| `derive <dir>` | Computes templates and contours. |
| `export <dir>` | Writes the map. `--force` replaces an existing map (old one as `.bak`). |
| `update` | Installs the newest freemapper release over this program. `--check` only checks, `--yes` installs without asking (needed without a terminal, so always for you), `--version <x.y.z>` picks a release, `--rollback` goes back to the replaced version, `--mode notify\|install\|off` stores what the daily check does. |

Output switches for `run` and `export`: `--ocad`, `--pdf`, `--pdf-candidates`, `--pdf-scale-bar`,
`--pdf-sources auto|full|credit-only`. All commands that download take `--cache-dir <folder>`.

Progress lines go to stdout. Lines starting with `warning:` (with `--lang de`: `Warnung:`) are warnings worth
passing on. On failure the program prints one line `error: <message>` to stderr (`Fehler: …`) and exits with code 1; a
usage error exits with code 2. Exit code 0 means success. `update --check` exits with 10 when a newer version
exists, also one that must be installed by hand, and with 0 when this one is current.

At most once a day, `init`, `select`, `fetch` and `run` also check for a newer release. When they find one, a
last line follows the command's output on stderr, also after an `error:` line:
`freemapper 0.12.0 is available (you have 0.11.0). Update with: freemapper update`. It does not change the exit
code. Pass it on and ask the person before you run `freemapper update --yes`. With `--offline`, with `CI` set or
after `freemapper update --mode off` there is no check. If the person chose mode `install`, freemapper installs
the update itself after a successful command and says so on stdout.

Workflow:

1. Ask what the map is for (school, park, sprint training) and which place.
2. Create the project. `init --place "<name>"` takes the best match with an area and does not list the others;
   if the name is ambiguous, ask the person, or use `init --osm way/<id>` when you know the object, or let them
   draw the area with `select <dir>` (it prints or opens the page address; wait until they say they saved).
3. Tell the person the download size before a large first download: the first project in a state
   downloads its OSM extract (about 50 to 500 MB, kept for 7 days); later projects are much smaller.
4. `freemapper run <dir>` with the outputs the person wants (`--pdf` to print, `--ocad` for OCAD, `--pdf-candidates`
   for a quick training map). It takes from one to several minutes, longer on the first download, which can
   be longer than your default command timeout: give the call a longer timeout or run it in the background and
   wait for it to end. Pass on warnings, not every progress line.
5. Tell the person which files are in the folder and what to check (see "What to tell the user").

The rules of the MCP section apply here too: an existing map is replaced only with `--force` after asking, and one
run per project folder at a time.

## Use it through MCP

For assistants that cannot run commands (Claude Desktop, for example). `freemapper mcp` is an MCP server on
stdin/stdout. Register it once with the absolute path:

```sh
claude mcp add freemapper -- /absolute/path/to/freemapper mcp          # Claude Code
```

```toml
# Codex: ~/.codex/config.toml
[mcp_servers.freemapper]
command = "/absolute/path/to/freemapper"
args = ["mcp"]
```

```json
{ "mcpServers": { "freemapper": { "command": "/absolute/path/to/freemapper", "args": ["mcp"] } } }
```

Optional arguments after `mcp`: `--cache-dir <folder>` (another download cache) and `--lang de|en` (language of the
server instructions, the tool and parameter descriptions and the texts the tools return; default:
`FREEMAPPER_LANG`, then the system language, then English). Tool names, parameter names, JSON field names and
states such as `running` or `finished` are the same in both languages. This file quotes the English texts; pass
`--lang en` to get exactly those.

### Tools

| Tool | Parameters | What it does |
|---|---|---|
| `find_place` | `query` | Searches a school, park or other place by name (OpenStreetMap Nominatim). Use a result with `has_area`; its `osm_id` becomes the map area. |
| `create_project` | `dir`; `osm_id` or `geojson`; optional `part`, `kind`, `scale`, `title` | Creates the project folder with `freemapper.toml`. `kind`: `schoolyard` (default) or `sprint`. `scale`: schoolyard 1000; sprint 4000 (default) or 5000. For 1:3000 use 4000; freemapper-print enlarges when printing. |
| `estimate_download` | `dir` | What `start_pipeline` will download, with sizes and what is already cached. |
| `start_pipeline` | `dir`; optional `force`, `offline`, `ocad`, `pdf`, `pdf_candidates`, `pdf_scale_bar`, `pdf_sources` | Fetches, derives and writes the project in the background. Returns a job id. |
| `project_status` | `job_id` | `running`, `finished` or `failed`, with the last progress line or the error. |
| `list_layers` | `dir` | The layers of a project: map, candidates, raster templates. |
| `preview_layer` | `dir`, `layer` | A PNG preview (at most 800 px) of `orthophoto`, `hillshade`, `slope` or `ndsm`. |
| `open_area_selector` | `dir`; optional `kind`, `scale`, `title`, `open_browser`, and the same output switches as `start_pipeline` | Opens a page in the browser on this computer where the user draws or picks the area and saves it. Returns the address. |
| `check_update` | none | Whether a newer freemapper release exists: version, release date, release notes link and download size, or that this one is up to date. Installs nothing. |
| `install_update` | `confirm` | Installs the newest release over the program the server runs from. Without `confirm: true` it installs nothing and returns an error. The new version applies once the MCP server is restarted. |

### Workflow

1. Ask what the map is for (school, park, sprint training) and which place. Then `find_place`.
2. If one result with an area fits, `create_project` with its `osm_id`. If several fit, ask the user. If none fits
   or the user wants to draw the area, call `open_area_selector`, give the user the address and wait until they
   say they saved; saving creates the project.
3. `estimate_download` and tell the user the size. The first project in a state downloads the state's OSM extract
   (about 50 to 500 MB, kept for 7 days); later projects in the same state are much smaller. Wait for an okay
   before a large download.
4. `start_pipeline`. Set `pdf: true` if the user wants something to print, `ocad: true` if they edit in OCAD,
   `pdf_candidates: true` for a quick training map that draws every uncertain object too.
5. Poll `project_status` every few seconds until `finished` or `failed`. A run takes from one to several minutes,
   longer on the first download. Pass the progress on in short lines, not every line.
6. On `finished`: `list_layers`, show one or two previews with `preview_layer`, and tell the user which files are in
   the folder and what to check (see "What to tell the user").

Rules:

- Paths must be absolute or start with `~`. Relative paths and the home folder itself are refused.
- One pipeline runs at a time per server, and one run per project folder across all processes. A second run on
  the same folder fails until the first ends; a lock left by a crashed run is taken over after 60 seconds.
- An existing map is kept, because it may hold the user's own work. Pass `force: true` only after asking; the old
  file stays as `<name>.omap.bak`.
- The server checks for a newer release once a day. When it knows one, the next tool result carries one extra
  text block at its end, the note `Note for the person: freemapper 0.12.0 is available (this is 0.11.0). Ask them
  before calling install_update.` It comes once per server process. Calling `check_update` or `install_update`
  takes its place: after a successful call of either the note does not come; after a failed one it still can.
  Pass the note on to the person.
- Never install without the person's yes. Ask, and call `install_update` with `confirm: true` only after they
  agreed. The server never installs on its own.
- After `install_update` succeeded, the server still runs the old version. Restart the freemapper MCP server, or
  ask the person to restart the assistant (Claude Desktop: quit and open it again).
- A note that says `install_update cannot install it` means the new release must be installed by hand from the
  linked release page (see "Install").
- `offline: true` works only with data fetched before into the cache.
- Errors come back as one sentence meant for the user. Relay it; do not retry the same call unchanged.

## What lands in the project folder

- `<name>.omap`: the map for OpenOrienteering Mapper, with the finished map objects and the peripheral information.
- `templates/`: `orthophoto.jpg`, `hillshade.png`, `slope.png`, `ndsm.png` with world files, and `candidates.omap`.
- `ocad/` (with `ocad`): `<name>.ocd`, `<name>-candidates.ocd` and the templates.
- `<name>.pdf` (with `pdf`) and `<name>-candidates.pdf` (with `pdf_candidates`).
- `ATTRIBUTION.txt`: every data source with its licence.
- `freemapper.toml`: map area, kind, scale and the data used. `data/`: the downloaded features.

The cache is shared by all projects: macOS `~/Library/Caches/freemapper`, Linux `~/.cache/freemapper`, Windows
`%LOCALAPPDATA%\freemapper`. It can grow to several GB; check free disk space before the first run in a new state.

## What to tell the user

- Open the `.omap` with OpenOrienteering Mapper (free, all systems). Templates can be switched on there.
- The map holds only what the data shows reliably. Everything uncertain (furniture, barriers, many paths,
  contours, out-of-bounds areas) is in the candidates map and must be checked against the orthophoto and in the
  terrain before it is copied into the map. The candidates PDF draws all of it and is for quick training only.
- The OCAD files carry placeholder symbols; in OCAD the user loads the official symbol set, which replaces them by
  number. The OCAD output has not been checked in OCAD yet.
- The maps carry the credit lines the data licences require. Keep them when the map is passed on. With
  `pdf_sources: "credit-only"` the PDF shows only the freemapper line, and the user must name the sources in
  another way (ATTRIBUTION.txt lists them).

## Troubleshooting

- "outside the supported states": the area is in a state or country without data sources. Nothing to fix on the
  agent's side.
- macOS "cannot be opened" or "killed": remove the quarantine flag (install step 4).
- A run fails with a download error: services of the states are sometimes down. Try again later; `offline` with
  data fetched before still works.
- "another freemapper run is using this folder" or "a run is already going on": another run holds the project
  folder. Wait for it, or use another folder.
- "map already exists": the folder has a map, which may hold the user's work. Ask before passing `force`.
- "cannot write to the program folder": the update cannot replace the program there and does not ask for
  administrator rights. Unpack the new release by hand, or move freemapper to a folder the person owns and
  register the new path.
- "the release could not be verified": the signature does not match; nothing was installed. Do not work around
  it. Tell the person and try again later.

## The freemapper suite

The `.omap` and the `freemapper.toml` that freemapper writes are read by the other programs of the suite. Each has
its own AGENTS.md:

- [freemapper-print](https://github.com/HenryJobst/freemapper-print/blob/main/AGENTS.md): print-ready PDFs of the
  map (sheet, margins, scales such as 1:3000 by enlargement).
- [freemapper-courseplan](https://github.com/HenryJobst/freemapper-courseplan/blob/main/AGENTS.md): course planning
  on the map.

## Working on the freemapper code

This section is for coding agents in a clone of the repository, not for using the program.

- Build and check: `cargo build --release --locked`, `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked`. The toolchain is
  pinned in `rust-toolchain.toml`.
- Source code and technical docs are English; terms follow [`docs/glossary.md`](docs/glossary.md). User docs are in
  German and English. German text says "Bundesland" for a state.
- Decisions are recorded in [`docs/adr/`](docs/adr/README.md); every change gets a changelog entry in
  `CHANGELOG.md` and `CHANGELOG.en.md`.
- Shared code comes from [freemapper-core](https://github.com/HenryJobst/freemapper-core), pinned by tag.
