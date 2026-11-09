# 0060. Writes never follow links

Date: 2026-11-09
Status: Accepted

Moved to freemapper-core ADR 0009; freemapper uses the crate `freemapper-fs` since v0.7.1 (ADR 0067).
The text below stays as decided. What it says of `fsutil` now lives in the crate, and `fsutil` keeps only
freemapper's wording of the refusals.

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
  link's target stays untouched. On Windows, std renames by handle (`FileRenameInfoEx` with replace) and falls
  back to `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING`; both replace a link at the target name. Cache files
  go through `write_cache_file`, the OSM extract download through `rename_cache_file`. The partial file is named `<file>.<process id>.<counter>.part` and opened
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
  it`. It then opens the source and checks the opened file again. It must be a regular file. The path,
  resolved after the open, must lie inside the project and name the opened file: `same-file` compares the
  two handles by volume serial number and file index on Windows, by device and inode on Unix. On Unix the
  device and inode must also match those of the path before and after the open. On Windows the source is
  opened with `FILE_FLAG_OPEN_REPARSE_POINT`, so a link or junction at the name is opened itself and refused.
  A source or folder swapped for a link between the check and the open, or swapped back right after it, is
  refused the same way. The workspace forbids `unsafe`, so the Windows calls come from `same-file`
  (`=1.0.6`, already in `Cargo.lock` through `walkdir`) and not from `windows-sys` directly. The OCAD export checks each template image and world file first and leaves a refused template out with
  `warning: …; the OCAD map gets no background from it`, so the export goes on without it. A map file that is
  a link stops `run --force` and `export --force` before the backup, with the error above and `; replace the
  link with the real file and run the command again`.
- Downloads that stream into a partial file of their own first remove an entry of that name, so they never
  write through a link either.
- Folders inside the cache may be links. The cache is the user's own, and moving a large part of it to another
  drive is a normal setup.
- No new dependency for `openat`-style access (such as `cap-std`). Paths are checked and then used by name
  again, so a race remains:
  - Fixed-name writes. The check resolves the folder of a file, and the rename uses the path again. Someone
    who swaps a project folder (`data`, `templates`, `ocad`) for a link between the two makes freemapper
    replace a file of that fixed name (`terrain.tif`, `hillshade.png`, `<name>.ocd`) in the folder the link
    points to. That can be any folder the user may write to, also one the attacker cannot write to.
  - Copies. The checks after the open compare the opened file with the file the path names inside the
    project, so a swapped file or folder is refused. What remains is a hard link: a file inside the project
    that is the same file as one elsewhere passes. An archive cannot carry such a link, and making one needs
    local access to the user's files.
  Both need someone who can write to the project folder while freemapper runs on it, and who times the swap
  to the moment of a write. A folder from a download or a shared archive cannot do that. We accept the race
  for this release. Users should not run freemapper on a project folder that others can write to while it
  runs; copy such a folder to a place only they can write to first.

## Consequences

- A shared or downloaded project folder can no longer make freemapper write outside it.
- A link at a file name inside the project is replaced by a regular file, silently.
- A project whose `data`, `templates` or `ocad` folder is a link to another drive no longer works. freemapper
  says which folder and stops.
- Copies no longer read through links, so a shared folder cannot pull a key or a password file into the
  project. Other reads still follow links (the map file, the stored data); what they read stays inside the
  process.
- Each file costs one flush to the disk.

The cleanup of stale `.part` files in project folders (`remove_stale_parts_inside`) checks the folder against the project root and skips link entries; only freemapper's own cache folders use the unchecked variant.

Source: security hotfix after v0.6.1; branch `hotfix-final-writes`.
