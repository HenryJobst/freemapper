# 0067. freemapper uses shared crates from freemapper-core

Date: 2026-11-09
Status: Accepted

## Context

freemapper, freemapper-courseplan and freemapper-print need some of the same code. freemapper-core holds it
as small crates, and an app pulls in only the ones it uses (freemapper-core ADR 0001). The first piece from
freemapper is `fsutil.rs`, the link-safe writes, copies and removals of ADRs 0059 and 0060. freemapper-core
0.1.0 took it over as `freemapper-fs`, with the same behaviour and two changes: the refusals no longer name an
app, and `remove_stale_parts_inside` returns a refused folder instead of printing a warning. The manual
(chapter 14) quotes the start of freemapper's messages, so their wording has to stay.

freemapper-core is not on crates.io. Its repository on GitHub is public.

## Decision

- freemapper depends on core crates through git, pinned to a release tag, in `[workspace.dependencies]`:
  `freemapper-fs = { git = "https://github.com/HenryJobst/freemapper-core", tag = "v0.2.0" }`. The crates use
  `freemapper-fs.workspace = true`. `Cargo.lock` records the commit of the tag, and CI and the release
  workflow build with `--locked` and need no token.
- Moving to a new core release changes the tag in that one place. The lock file diff may then touch only the
  core crates and what they bring in; anything else is undone.
- To change freemapper and a core crate together, freemapper-core lies next to freemapper
  (`../freemapper-core`) and the root `Cargo.toml` gets a temporary
  `[patch."https://github.com/HenryJobst/freemapper-core"]` entry with the path of the crate. It comes out
  before the merge. freemapper merges only against a released tag (freemapper-core ADR 0006).
- The two-week rule of ADR 0012 does not apply to the suite's own crates: freemapper may pin a core release
  on the day it appears (freemapper-core ADR 0004). Third-party crates that core brings in follow the rule,
  checked in core.
- Who owns what. freemapper maintains the core crates it brought in, `freemapper-fs` and the planned
  `freemapper-georef` (freemapper-core ADR 0005). A change to their behaviour is made and tested in core first.
  A change to a crate another app owns is agreed with that app first. freemapper keeps a thin module for what
  only it needs: user-visible wording, printed warnings, and code tied to `fetch` or the project types. For
  `freemapper-fs` this is `fsutil.rs`. It puts the two refusals back into freemapper's words ("so freemapper
  writes nothing there", "so freemapper does not copy it"), prints
  `warning: not cleaning stale parts in <folder>: <reason>`, and names the partial files that downloads stream
  into (`fresh_part_name`). Calls that cannot be refused go to `freemapper_fs` directly.
- The crate's guarantees are tested in core. freemapper tests its wording, its warning and the project
  behaviour with links.

## Consequences

- One copy of the link-safe file code serves all three apps. A fix reaches freemapper when it moves the tag.
- A core change takes two steps: a core release, then the tag in freemapper.
- The first build fetches freemapper-core from GitHub. After that, cargo builds from its local git checkout,
  also offline.
- If core changes a refusal text, `fsutil` no longer recognises it and freemapper's tests on the wording fail.
- Cost if wrong: if the GitHub repository became unreachable, new builds would fail until freemapper vendored
  the crates or core moved to crates.io.

Source: user request of 2026-11-09 (move freemapper onto `freemapper-fs` of freemapper-core v0.2.0);
freemapper-core ADRs 0001, 0004, 0005, 0006, 0008, 0009; branch `core-fs`.

Update 2026-11-10: freemapper also uses `freemapper-georef` (core ADR 0015) and both crates are pinned to freemapper-core v0.5.1. `crs.rs` and `tiles.rs` re-export the crate; the 12 tests of those modules moved with the code. Branch `core-georef`.

Update 2026-11-11: freemapper also uses `freemapper-web` (core ADR 0020) for the area selection page. It, `freemapper-fs`, `freemapper-georef` and the test-only `freemapper-contract` now all point at the one tag v0.11.0, so the lock file holds one copy of each core crate. Before, v0.5.1 and v0.8.0 lay side by side, with two copies of `freemapper-omap`. Pin all core crates to a single tag from now on. The contract tests of ADR 0089 pass unchanged. Stage 1 of ADR 0090.

Update 2026-11-11 (ADR 0095): all core crates point at v0.18.0. `freemapper-contract` is now also a normal dependency of the library, without default features, for the map kinds and the reserved words; only the tests take its fixtures and with them `freemapper-omap`.
