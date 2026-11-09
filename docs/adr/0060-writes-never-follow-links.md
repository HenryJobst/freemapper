# 0060. Writes never follow links

Date: 2026-11-09
Status: Accepted

## Context

ADR 0059 made sure that names from the project file never become paths. The folder itself can still carry
links, because people share project folders and download examples. Up to 0.6.1, freemapper wrote through them:

- `ATTRIBUTION.txt` on every `run` with an existing map, without `--force`;
- the backups `<name>.omap.bak` and `ocad/<name>.ocd.bak` with `--force`. They copied the project's own map,
  so whoever built the folder chose what landed in the link's target, for example `~/.zshrc`;
- the image and world-file copies into `ocad/`;
- every file with a fixed name in `data/`, `templates/` or `ocad/` when that folder was a link.

Most other files already went through a partial file and a rename. The partial file had a predictable name
(`<file>.<process id>.part`), and `std::fs::write` follows a link of that name.

## Decision

- Every file freemapper writes, in the project and in the cache, goes through `fsutil`. It writes a new
  partial file next to the target, renames it over the target name and so replaces a link at that name. The
  link's target stays untouched. The partial file is named `<file>.<process id>.<counter>.part` and opened
  with `create_new`, which fails on any existing entry, a link included. A taken name is skipped. freemapper
  flushes the partial file to the disk before the rename.
- Project files go through `write_inside` and `copy_inside`, which take the project folder. They refuse a
  file whose folder leads out of the project folder once links are resolved, before any folder is created.
  The error names the file: `<path>: a link leads out of the project folder, so freemapper writes nothing
  there`.
- Some writers do not know the project folder: the downloads into `data/` and the derived templates. For them
  `fetch`, `derive`, `run` and a new map area from `select` first check `data/` and `templates/`, and the OCAD
  export (`--ocad`) checks `ocad/`. A linked folder stops the command with the same message, naming the folder.
  `data/berlin` keeps its own check and warning (ADR 0059).
- Downloads that stream into a partial file of their own first remove an entry of that name, so they never
  write through a link either.
- Folders inside the cache may be links. The cache is the user's own, and moving a large part of it to another
  drive is a normal setup.

## Consequences

- A shared or downloaded project folder can no longer make freemapper write outside it.
- A link at a file name inside the project is replaced by a regular file, silently.
- A project whose `data`, `templates` or `ocad` folder is a link to another drive no longer works. freemapper
  says which folder and stops.
- Reads still follow links. A linked map file is copied into its backup, and a linked template image into
  `ocad/`. The data stays in the project folder.
- Each file costs one flush to the disk.

Source: security hotfix after v0.6.1; branch `hotfix-final-writes`.
