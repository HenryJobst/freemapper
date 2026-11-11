# 0094. Self-update from signed releases

Date: 2026-11-11
Status: Accepted (app side merged; the signed release workflow follows once the release key exists)

## Context

freemapper ships as an archive that a person unpacks once and registers with an AI assistant by its absolute path.
Releases come every few days, and nothing tells the person that a new one exists. Updating means a new download,
unpacking and a new registration. The release checksums sit next to the archives, so they prove integrity, not
origin. The user asked for an auto-update feature on 2026-11-11.

## Decision

- `freemapper update` replaces the running executable in place with the newest release: download, check, swap by
  two renames, run `--version` on the new file, keep the old one as `freemapper.previous` for `--rollback`. The
  path stays the same, so the MCP registration keeps working.
- Every release publishes the bare executables as `.gz` and one manifest `freemapper-update.json` with sizes and
  SHA-256 sums, signed with minisign. The key is made by the user, not by an agent; its public half is compiled in,
  as a list so it can be rotated. Nothing unsigned is installed.
- The program finds the manifest at `releases/latest/download/freemapper-update.json`, without the GitHub API.
- A check runs at most once a day, after `init`, `select`, `fetch` or `run`, in the background with a 3-second
  budget, and prints one line. The MCP server checks too and adds a note to its first tool result; it installs
  only through the `install_update` tool after the person agreed.
- Mode `notify` (default), `install` (installs after a command on its own) or `off`, stored in
  `<config_dir>/freemapper/settings.toml`, overridable with `FREEMAPPER_UPDATE`.
- Builds from source never update themselves; release builds carry `FREEMAPPER_DIST=github-release`.
- The pure parts (manifest, signature, versions, swap, settings) go into a new crate `freemapper-update` in
  freemapper-core; the network stays in the app (core ADR 0007). One new dependency: `minisign-verify` 0.3.0.

## Consequences

- People get updates with one command or none at all, and a bad release can be undone with one command.
- The release workflow cannot sign before the user has made the key; until then no release ships the updater.
- freemapper-print and freemapper-courseplan can use the same crate with a small app part.
- Cost if wrong: an update that breaks reaches everyone in `install` mode at once (rollback and the post-release
  test limit it); a lost signing key without a successor forces one manual update; GitHub changing its redirect
  breaks the check until the program falls back to the API.

Addendum (2026-11-11, release workflow): `min_self_update_from` carries over from the previous release's signed
manifest and is raised only by hand, through the pin `[workspace.metadata.update] min_self_update_from` in
`Cargo.toml` that `.github/scripts/update-manifest.sh` reads; the first updater release takes its own version.

Source: user request of 2026-11-11; spec `docs/superpowers/specs/2026-11-11-self-update-design.md`; plan
`docs/superpowers/plans/2026-11-11-self-update.md`; ADRs 0012, 0067, 0088, 0092; freemapper-core ADRs 0005, 0006,
0007, 0009.
