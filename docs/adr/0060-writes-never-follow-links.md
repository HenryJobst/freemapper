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
- Every project file goes through `write_inside`, `write_files_inside` or `copy_inside`, which take the
  project folder. This includes the downloads into `data/` and the derived templates: the source and derive
  writers get the project folder as a parameter. Before each write, freemapper resolves the folder of the file
  and refuses it if it leads out of the project folder, before any folder is created and again once it exists.
  The error names the file: `<path>: a link leads out of the project folder, so freemapper writes nothing
  there`. Removing a stale project file follows the same check.
- `fetch`, `derive`, `run` and a new map area from `select` also check `data/` and `templates/` once at the
  start, and the OCAD export (`--ocad`) checks `ocad/`. A linked folder then stops the command before any
  download, with the same message naming the folder. `data/berlin` keeps its own check and warning
  (ADR 0059).
- A copy reads only files of the project itself. `copy_inside` refuses a source that is a link or whose folder
  leads out of the project folder: `<path>: a link or outside the project folder, so freemapper does not copy
  it`. The OCAD export checks each template image and world file first and leaves a refused template out with
  `warning: …; the OCAD map gets no background from it`, so the export goes on without it. A map file that is
  a link stops `run --force` and `export --force` before the backup, with the error above.
- Downloads that stream into a partial file of their own first remove an entry of that name, so they never
  write through a link either.
- Folders inside the cache may be links. The cache is the user's own, and moving a large part of it to another
  drive is a normal setup.
- No new dependency for `openat`-style access (such as `cap-std`). The check resolves the folder by its path,
  and the rename uses the path again. A local attacker who swaps a project folder for a link between the two
  can still win that race. We accept this: it needs write access to the project folder while the run is
  going on, and such an attacker can already change the project's files directly. The case this ADR closes is
  the folder that arrives with links already in it.

## Consequences

- A shared or downloaded project folder can no longer make freemapper write outside it.
- A link at a file name inside the project is replaced by a regular file, silently.
- A project whose `data`, `templates` or `ocad` folder is a link to another drive no longer works. freemapper
  says which folder and stops.
- Copies no longer read through links, so a shared folder cannot pull a key or a password file into the
  project. Other reads still follow links (the map file, the stored data); what they read stays inside the
  process.
- Each file costs one flush to the disk.

Source: security hotfix after v0.6.1; branch `hotfix-final-writes`.
