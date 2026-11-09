# 0059. Project files never name paths

Date: 2026-11-09
Status: Accepted

## Context

freemapper stores each extra Berlin layer in `data/berlin/<file>.geojson` and records it in `freemapper.toml`
under `berlin_layers`, keyed by the file name. Up to 0.6.0, `fetch` deleted stored layers by the keys it read
from the project file. A project file is plain text that people share and download. A key like `../../../x`
made `fetch` delete `x.geojson` outside the project folder.

The other records in the project file do not reach a path. Tile names are parsed into numbers and the path is
built from them, template paths are constants, and the map file is named after the project folder. Names from
servers that end up in cache paths (Schleswig-Holstein, North Rhine-Westphalia, Rhineland-Palatinate, Hamburg,
Hesse) are already checked against strict patterns or replaced by names freemapper builds itself.

## Decision

- freemapper builds a file path only from names it knows. For the Berlin layers these are the `file` names of
  the static table `berlin::EXTRA_LAYERS`. A record key is first looked up in the table, and only the table's
  name goes into the path. `berlin_layer_path` takes a table entry, not a string, so the compiler holds this.
- Loading a project drops every `berlin_layers` entry whose key is not a plain file name (path separator, `..`,
  drive prefix, absolute path, NUL or another control character) or not a layer of the table. The rest of the
  project loads. `fetch` and `derive` print `warning: freemapper.toml: ignored the Berlin layer entry "…": …`
  for each, with the key escaped. The next save leaves the entry out.
- Before writing or removing a stored layer, freemapper resolves the folder of the file. If a link leads it
  out of the project folder, nothing is written or removed. A write then ends as
  `warning: Berlin … unavailable (data/berlin leads out of the project folder); layer skipped`.
- New records in the project file follow the same rule. A name read from a record or a server goes into a
  path only after a lookup in a fixed table or a strict pattern check.

## Consequences

- A crafted project file can no longer make freemapper delete or write files outside the project.
- A layer file left from a key that a later version renames or drops stays on disk. freemapper no longer
  deletes it, since it does not know the name. No layer has been renamed so far.
- A project whose `data/berlin` folder is a link to another drive gets no Berlin layers. That setup is unusual,
  and the warning names the reason.

Source: security hotfix after v0.6.0; branch `hotfix-layer-paths`.
