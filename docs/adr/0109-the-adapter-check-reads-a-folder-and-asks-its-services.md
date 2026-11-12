# 0109. The adapter check reads a folder and, when asked, its services

Date: 2026-11-12
Status: Accepted

Part of stage E of [0082](0082-region-adapters-trait-and-manifests.md) (task E2). Builds on
[0106](0106-requests-go-to-public-addresses-and-declared-hosts.md) and
[0107](0107-adapter-manifests-format-1-as-built.md). Number 0108 is taken by work that is not on `main` yet.

## Context

Someone who writes an adapter needs to know two things before passing it on: would freemapper load the folder,
and do the services deliver what the manifest says. Until now the first answer came as a warning on the next run,
and the second only from a map made in the region.

## Decision

- `freemapper adapters check <dir>` checks the folder of one adapter. `adapter check` is the same command.
- Without a switch it asks no server. It reads the folder with the code of the loader, so what passes the check
  loads from an adapter folder: the folder is no link and has the name of the id, the manifest is valid, the
  boundary files are valid, every backend is one this freemapper has, and the manifest does not need a newer
  freemapper.
- Beyond the loader it warns: the manifest has no `last_verified` or one older than twelve months, the id or the
  object-id prefix is that of a built-in adapter, the boundary encloses less than 1 km² or more than 2 million
  km², the adapter lies on the ground of a built-in one. A warning does not fail the check.
- `--live --bbox minLon,minLat,maxLon,maxLat` also fetches a real project for that box, with the checked adapter
  as the only one. The adapter counts as one from a folder, so every request stays on the hosts the manifest
  declares, redirects included. The box must lie inside the boundary. The report has one line per capability:
  rasters, ALKIS buildings, ALKIS land use, the full ALKIS, each tree cadastre and the state of the OSM extract.
  The extract itself is not downloaded.
- Exit code 0: the adapter passes. 1: it has findings, or a service failed. 2: the check could not run (no such
  folder, a box outside the boundary).
- `--json` prints the same as an object with `passed`, `adapter`, `findings` (each with `severity`, `code`,
  `file`, `line`, `key`, `message`), `live` and `live_warnings`.

## What differs from the plan

- The plan had a switch `--fixtures`: replay a fetch, a derive and a build from recorded answers in the folder.
  It is not built. No folder has such a recording, and the tool that would write one (`adapter record`) was
  deferred on 2026-11-11. The first state added as a folder decides what a recording needs; the replay code is
  ready in `freemapper_sources::replay`.
- The plan named a check of `rules.toml`. freemapper does not read the rules of an adapter (0107), so the check
  says that a `rules.toml` is not read.
- The findings of a manifest are English in a German run. They carry a stable code.

## Consequences

- The manifest folders of the eleven built-in states pass the check; a test holds that. Run with `--live`, the
  check holds a state to its declared hosts. The live runs that 0106 waited for were made with it, and the
  pipeline holds the built-in states to their hosts since then.
- A live check costs what a fetch of the box costs. For a state with tiles that is whole tiles in the cache.
- An adapter is checked alone. An object at a border that needs the neighbour's service is not covered.
