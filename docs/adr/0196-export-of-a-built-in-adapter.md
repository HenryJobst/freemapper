# ADR 0196: Export of a built-in adapter and the guide to replacing it

## Status

Accepted, 25 November 2026. Extends [ADR 0188](0188-a-folder-adapter-replaces-a-built-in-one.md) and
[ADR 0189](0189-permit-sources-as-manifest-data.md).

## Context

ADR 0188 lets a folder in the person's adapter folder replace a built-in adapter, so that a moved service is
corrected without a release. Two things were missing for that to be usable. A person had to copy the manifest
folder of a state out of the source tree of the repository, which a person with a release archive does not have,
and some states need files beside the manifest (the boundary, the boundaries of OSM extracts). And the way was
written down in one bullet of the contributor guide only, not for the people and agents who meet a moved service.
Since stage 2 of issue #273 the addresses of the permit notes, the cadastres and the rasters of every state that
a manifest can describe are in the manifests, so the export is now worth something.

## Decision

`freemapper adapters export <id> <dir>` writes the folder `<dir>/<id>` with the `manifest.toml` of the built-in
adapter and every file the manifest names, byte for byte the files the program carries. It never overwrites: the
folder `<dir>/<id>` must not exist or must be empty. A manifest that declares its orthophoto as a service (Saxony-Anhalt's) needs a provider of its own to load as a
replacement (ADR 0188), so the export names the state's id as its provider; that is the one difference from the
built-in text, and the downloads of the replacement go to a cache folder of its own. An adapter that no manifest of the program declares (Poland's,
which is code) has nothing to export, and the command says so. `adapters check` on the export passes with the one
warning `check.built-in-id`, which is the point: the id is built in, so the folder will replace it. The scope of the
replacement stays that of ADR 0188: the person's own adapter folder only, never a project folder.

A replacement can fall behind. When the built-in adapter has a higher `revision` than the folder that replaces
it, the loader adds the note `adapter.replacement-outdated` at each start and `adapters check` the warning
`check.built-in-newer`; both say to export again and carry the changes over. The note `adapter.replaces-built-in`
now also says that the credit and the licence of the manifest are the person's to keep right and that addresses
in the program's code stay as they are.

The way is documented in the manual (chapter 13, "Correcting an address yourself"), in the contributor guide
(chapter 10), in the reference of the findings and in `AGENTS.md`, with the steps, the limits and the safety rules.
`AGENTS.md` tells an agent to propose the change and write it only after the person agreed, and the troubleshooting
entries about the message of a refused host (manual and `AGENTS.md`) point to it.

## Consequences

- A person with a release archive can correct a moved address in a few places: export, edit, check, move the folder.
- What a manifest does not declare cannot be replaced: Berlin's layers fetched by code, tree cadastres, point clouds,
  the permit sources no manifest table describes. The documentation says so in each place; the warning at start says
  it in short.
- A replacement does not follow the program's updates for its state. The revision comparison is the only prompt; a
  replacement that stays for years is the person's to keep alive.
- The export has no `--force`. A person who wants it again deletes or renames the folder.

## Addendum: `freemapper sources` (issue #273, stage 3)

The export and the edit by hand are the long way. `freemapper sources` is the short one, and the one an agent can
script. `sources list` and `sources get` read the addresses of the manifest in force for an adapter (the folder of
the person when it loaded, else the built-in manifest); an address is a string value that starts with `https://`
or an entry of `[net] hosts`, named by its path in the file (`permits.wfs[0].url`). `sources set <adapter> <path>
<url>` replaces that one value in the text of `manifest.toml` and leaves every other byte, comments included. It
writes only into the person's own adapter folder (ADR 0188), makes the folder from the built-in manifest as the
export does when it is not there, and runs the full manifest check on the result before it writes: a change that
would not load writes nothing. A new address on a host the manifest does not declare is refused; `--add-host`
declares it and the command says so, so that no change frees a host unnoticed. Credit and licence are not touched
and not checked beyond the manifest check; the command says they stay the person's to keep right. `sources diff`
compares the addresses of the folder with the built-in manifest, `sources check` is `adapters check` on the folder
of that id, and `sources schema` prints the keys that hold an address, as a JSON Schema with `--json`.

Not in this stage: `sources reset` (deleting the folder does it, and the manual says so) and the signed list of
corrections (stage 4).
