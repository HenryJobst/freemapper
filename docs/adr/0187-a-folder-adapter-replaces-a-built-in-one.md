# ADR 0187: A folder adapter replaces a built-in one

Date: 2026-11-22

Status: Accepted

Refs: issue #273, ADR 0082, ADR 0110.

## Context

Services of the states move, and until now every correction of an address in a manifest needed a new release. An adapter folder could not help: `Registry::add` refused an id that was taken, so a folder with the id `hamburg` was left out with `adapter.duplicate`. Only `adapters allow-host` worked without a release, and only for a new host.

## Decision

An adapter from a folder (the person's adapter folder, or a project folder with `--adapters-from-project`) with the id of a built-in adapter takes its place, once. The registry marks the built-in ids when the program has filled it (`mark_builtin`); `replace` puts the new adapter where the old one stood, so the order that decides ties on a border stays. A second folder with the same id, or a folder with an id that came from another folder, is still `adapter.duplicate`.

The replacement is announced, never silent: the loader adds the note `adapter.replaces-built-in` (shown as a warning at each start, named in `adapters check` as `check.built-in-id`), `adapters list --json` says `replaces_built_in`, and `adapters list` shows the source `user` or `project`. The host rules do not weaken: the replacement is held to the hosts its own manifest declares, as every folder adapter is, and the hosts a person allowed for the built-in adapter do not carry over. A replacement does not count as sharing ground with the adapter it replaces, and its provider may be the provider of the adapter it replaces.

Only what a manifest declares changes. Addresses that still sit in the code of a state (permit sources, tree cadastres, point clouds) are not in the manifest; they move with stage 2 of issue #273.

## Consequences

- A moved service that a manifest declares is corrected by a folder, without a release: the built-in manifest is the template.
- A newer freemapper that changes the built-in state does not reach a region whose folder replaces it; the note says so.
- The other stages of #273 follow: addresses from code into manifests, commands to maintain the folder, a signed list of corrections.
