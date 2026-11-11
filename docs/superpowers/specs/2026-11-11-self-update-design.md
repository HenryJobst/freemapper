# Self-update design

Status: proposed · Date: 2026-11-11 · Terms follow [`docs/glossary.md`](../../glossary.md). Decision record:
[ADR 0094](../../adr/0094-self-update-from-signed-releases.md). Plan:
[`2026-11-11-self-update.md`](../plans/2026-11-11-self-update.md).

The user asked for "a plan for an auto-update feature" on 2026-11-11 and wants decisions made without follow-up
questions. Every call below is marked as the designer's call; section 8 lists them with their cost if wrong.

## 1. Intent

freemapper ships as an archive that a person unpacks once, often with an AI assistant's help, and then registers
with that assistant by its absolute path. New releases come every few days. Today nothing tells the person that a
new version exists, and updating means downloading, unpacking and re-registering by hand. Most users are club
volunteers, not developers; many reach freemapper only through an assistant.

The feature: freemapper notices a new release, says so in one line, and replaces itself in place on request, or
on its own if the person chose that. The path stays the same, so the MCP registration keeps working.

### Goals

- One command, `freemapper update`, that installs the newest release over the running program, on Windows, macOS
  and Linux, without admin rights when the folder belongs to the user.
- A notice when a newer release exists, at most once a day, never slowing a command down.
- An opt-in mode that installs updates on its own (the "auto" in auto-update).
- Authenticity: only a release signed with the project's key is installed. A compromised download host or a
  swapped archive must not lead to running foreign code.
- A way back: the previous version stays and `freemapper update --rollback` restores it.
- The same mechanism for freemapper-print and freemapper-courseplan later, from shared code in freemapper-core.

### Non-goals

- Package managers (Homebrew, winget, apt). If they come, they update the program and the self-update steps aside.
- Updating a build from source. It says how to update with git and cargo instead.
- Delta updates, pre-release channels, staged rollouts.
- Updating the documentation files in the unpacked folder (README, manual, AGENTS.md); the notice links to them
  online. (Designer's call, D7.)

### Anti-goals

- Telemetry. The check sends no id, no project data, no personal data; only the fixed `User-Agent`.
- An update during a run, inside the MCP stdio stream, or on a machine that said `--offline`.
- A silent install by default. Default is "notify".
- Running an unsigned or unverified binary, even "just to check its version".

## 2. What exists

- Release workflow `.github/workflows/release.yml`: five targets (`linux-x86_64` and `linux-arm64` musl,
  `macos-arm64`, `macos-x86_64`, `windows-x86_64`), archives `freemapper-<tag>-<platform>.tar.gz|.zip` with a
  `.sha256` each. The checksums sit next to the archives, so they prove integrity, not origin.
- `https://github.com/HenryJobst/freemapper/releases/latest/download/<file>` redirects to the newest release's
  asset of that name (checked 2026-11-11), so a fixed file name needs no GitHub API call and no rate limit.
- HTTP through `ureq` 3.4.2 with a fixed `User-Agent` naming the program and repository (`fetch.rs`).
- `flate2` 1.1.11 is already in `Cargo.lock`; `zip` 9.0.5 and `sha2` are direct dependencies.
- freemapper-core: shared crates pinned by tag (ADR 0067), no network in core (core ADR 0007), link-safe writes
  in `freemapper-fs` (core ADR 0009), the session that adds a crate owns it (core ADR 0005).
- `unsafe_code = "forbid"` in the workspace.

## 3. Design

### 3.1 Release side

Every release gets three more kinds of assets:

- `freemapper-<tag>-<platform>.gz`: the bare executable, gzip-compressed. The updater never parses tar or zip.
- `freemapper-update.json`: the update manifest, one per release, with a fixed name:

  ```json
  {
    "format": 1,
    "app": "freemapper",
    "version": "0.11.0",
    "released": "2026-11-20",
    "notes": "https://github.com/HenryJobst/freemapper/releases/tag/v0.11.0",
    "min_self_update_from": "0.11.0",
    "assets": {
      "macos-arm64": { "url": "https://github.com/HenryJobst/freemapper/releases/download/v0.11.0/freemapper-v0.11.0-macos-arm64.gz",
                       "size": 12345678, "sha256": "…", "exe_sha256": "…" }
    }
  }
  ```

- `freemapper-update.json.minisig`: a minisign signature (Ed25519) of the manifest bytes.

The release job builds the manifest from the five `.gz` files after all builds finished, signs it with the secret
key from the repository secrets `MINISIGN_SECRET_KEY` and `MINISIGN_PASSWORD`, and refuses to publish without a
valid signature. The public key is committed as `assets/update/minisign.pub` and compiled into the program. The
program holds a list of accepted keys, so a key can be rotated: a release signed with the old key adds the new key
first.

Signing only the manifest is enough: the manifest pins size and SHA-256 of each `.gz` and of the unpacked
executable.

### 3.2 The check

`check_for_update(current, fetch) -> Result<Option<Available>, UpdateError>`:

1. GET `…/releases/latest/download/freemapper-update.json` and `….minisig` (redirects only to `github.com`,
   `release-assets.githubusercontent.com` and `objects.githubusercontent.com`; each answer capped at 64 KiB).
2. Verify the signature against the built-in keys. A bad signature is an error that says so; it is never a "no
   update".
3. Parse with `deny_unknown_fields` except inside `assets` (new platforms may appear). `app` must equal
   `freemapper`. `format` must be 1.
4. Compare versions (three integers, hand-written; pre-release suffixes are refused). Newer than current gives
   `Available { version, notes, asset }` for this platform; equal or older gives `None`.

### 3.3 When the check runs

- Explicitly: `freemapper update --check` (prints the result, exit 0 if current, 10 if an update exists).
- Implicitly: after a command that uses the network anyway (`init`, `select`, `fetch`, `run`) finished, if the
  last attempt is older than 24 hours. `derive` and `export` work offline and send no request today, so they do not
  check either (D11). The check runs on a background thread started with the command, with a
  3-second total timeout counted from the start: when the command ends first, it waits for the rest of the 3 s
  (so a very fast command can take up to 3 s longer); a check not done by then is dropped and tried next time. The result
  prints one line on stderr after the command's own output:
  `freemapper 0.11.0 is available (you have 0.10.0). Update with: freemapper update`. Translated (ADR 0092).
- Never: on stdout of `mcp` (the protocol; see 3.7 for the server), with `--offline` on the commands that have it
  (`fetch`, `run`, `select`), when `CI` is set, for `--help` and `--version`, for a build from source (3.6), or
  when the mode is `off`.
- `<cache>/update-check.json` holds the last attempt: when, outcome (`current`, `available` with version and
  notes, or `failed` with a short reason) and the program version that made it. A failed or dropped attempt
  counts too, so a machine without network asks once a day, not on every command (D12). Writes go through
  `freemapper-fs` create-new plus rename.

### 3.4 Modes

A small settings file `<config_dir>/freemapper/settings.toml` holds `[update] mode = "notify" | "install" | "off"`,
written by `freemapper update --mode <mode>`. `FREEMAPPER_UPDATE=off|notify|install` overrides it for one process.
Default `notify`. (Designer's call, D3: the first user setting freemapper stores; `dirs::config_dir()` is the
place the adapter design also uses.)

In `install` mode, the implicit check of 3.3 is followed by an install after the command finished, with a line
saying it did so and that the new version applies from the next start. It never installs before or during a
command, and never when the command failed.

### 3.5 The install

`freemapper update [--yes] [--version <x.y.z>]`:

1. Check (3.2). With `--version`, take that release's manifest (`…/releases/download/v<x.y.z>/freemapper-update.json`)
   and allow an older version only then.
2. If `min_self_update_from` is newer than the current version, stop with a message that links the release page
   (a release that changes the update path itself can require a manual step).
3. Show version, release date, notes link and download size; ask `[y/N]` unless `--yes` or stdin is not a
   terminal (then it needs `--yes`).
4. Download the `.gz` into a temp file in the folder of the running executable (so the final rename stays on one
   file system), checking size on the fly and SHA-256 at the end; decompress into a second temp file and check
   `exe_sha256`.
5. Set the executable bit (Unix).
6. Run `<temp file> --version` and compare with the manifest version; on a mismatch delete the temp files and
   report. This comes before the swap, so a broken binary never takes the place of the working one.
7. Swap: rename the running executable to `freemapper.previous` (Windows `freemapper.previous.exe`), then rename
   the new file to the executable's name. Windows allows renaming a running executable but not deleting or
   overwriting one; on Unix the running process keeps its inode. An older `previous` is first renamed to
   `freemapper.stale-<unix-seconds>[.exe]` (this works even while an old MCP server still runs from it), and every
   later start deletes `stale-*` files it can, silently. If the second rename fails, rename the previous one back.
   The swap is its own code in the core crate: `freemapper-fs` has create-new and a cache rename, but not a
   two-step swap with restore (D13).
8. Print `freemapper updated from 0.10.0 to 0.11.0` and the notes link.

`freemapper update --rollback` swaps `previous` and current back, after checking that `previous --version` runs.

The executable's path comes from `std::env::current_exe()`, canonicalised. If its folder is not writable, the
update stops with a message naming the folder (no sudo, no elevation). If the path is a symlink (for example
`~/.local/bin/freemapper` pointing into the unpacked folder), the target is replaced, not the link.

Another freemapper process (an MCP server, a `select` page) may still run the old version; that is fine and the
notice says a running MCP server needs a restart of the assistant to pick up the new version.

macOS: a file written by freemapper itself carries no quarantine flag, so Gatekeeper does not block the new binary
the way it blocked the first download. The arm64 binary is ad-hoc signed by the linker as today.

### 3.6 Builds from source

`release.yml` sets `FREEMAPPER_DIST=github-release` in the environment of the build step; `freemapper-cli`
reads it with `option_env!` (the updater code lives in that crate and the core crate, so no library crate needs
it).
Without it, `update` prints that this build came from source and how to update it (`git pull`,
`cargo build --release --locked`), and no implicit check runs. Package managers can set another value later to
switch the updater off.

### 3.7 MCP and the browser page

- MCP tools `check_update` (no parameters; the result of 3.2 as text) and `install_update` (`confirm: true`
  required, which the agent sets only after the person agreed). After an install the tool says the assistant must
  restart the freemapper server. The server instructions mention both. The edits to `mcp.rs` are agreed with the
  program-language session first.
- Many people reach freemapper only through an assistant, so the server checks too: at start it runs the daily
  check of 3.3 on a background thread (same cache, same 24 hours, nothing on stdout). When it knows a newer
  version, the first tool result of the session gets one extra line: "Note for the person: freemapper 0.11.0 is
  available (this is 0.10.0). Ask them before calling install_update." The mode `install` does not apply to the
  server; it never installs without the tool call (D14). AGENTS.md tells the agent to pass the note on and ask.
- The browser page shows a one-line notice with the command when the cached check (3.3) knows a newer version. No
  install button in the first version (D8).

### 3.8 Where the code lives

- freemapper-core, new crate `freemapper-update` (owned by the freemapper session, core ADR 0005): manifest types
  and parser, signature check (`minisign-verify`), version comparison, platform name, the swap and rollback, the
  settings file. No network (core ADR 0007): the crate takes a `trait UpdateSource { fn get(&self, url: &str,
  max_bytes: u64) -> Result<Vec<u8>, …>; fn download_to(&self, url: &str, size: u64, file: &mut File) -> … }` that
  each app implements with its own HTTP client. App name, repository and accepted keys come from the app. The crate
  exports `ALLOWED_HOSTS` (the three GitHub hosts) and `check_url(url)`; the app's source must build its agent with
  redirects off and follow them itself, at most three, each target passing `check_url` (D15). freemapper's general
  agents follow redirects to any host today, so the update source gets its own agent. A test-only constructor
  `UpdateSource` over a stub on 127.0.0.1 exists behind `#[cfg(test)]` and a `test-stub` feature, never in a
  release build.
- freemapper: `UpdateSource` over `HttpFetcher`, the CLI command, the background check, the MCP tools, the page
  notice, the release workflow.
- freemapper-print and freemapper-courseplan adopt it later with their own keys or the same key (their call).

### 3.9 Security

| Threat | Answer |
|---|---|
| Swapped asset on the download host | manifest signature plus SHA-256 of `.gz` and executable |
| Downgrade to an old, vulnerable release | a signed old manifest is refused unless `--version` asks for it |
| Manifest of another app or a future format | `app` and `format` checked; unknown top-level keys refused |
| Huge or endless answers | caps: 64 KiB manifest and signature, `size` from the manifest for the binary |
| Redirect to a foreign host | redirects only to the three GitHub hosts |
| Lost signing key | rotation through the key list; a lost key without a successor means a manual update once |
| Partial write, power loss | temp files, two renames, the old executable kept until the new one ran |
| Link tricks in the program folder | temp files created with `freemapper-fs` create-new, never following links (the executable's own symlink is resolved once, 3.5) |

### 3.10 Testing

- Unit tests in the core crate: manifest parsing (valid, unknown key, wrong app, format 2), version order, key
  rotation, signature good and bad (test key pair committed under `tests/`, never the real one), platform names.
- Swap tests in temp folders on all three CI systems, including a failing second rename and a new binary that
  prints the wrong version.
- An end-to-end test against a local stub server on 127.0.0.1 with a signed test manifest (the stub host is allowed
  only in tests).
- After each release: a job on the three systems downloads the previous release, runs `freemapper update --yes`
  and checks `--version`. It runs from the second release with the updater on.

## 4. Size

About 900 lines in the core crate, 500 in freemapper, 80 in the release workflow, plus docs. One new dependency,
`minisign-verify` 0.3.0 (2026-09-25, no dependencies of its own; ADR 0012 satisfied); `flate2` becomes direct at
the locked 1.1.11.

## 5. Human steps

The signing key is the one thing an agent must not make: the user creates it with `minisign -G` on their own
machine, adds `MINISIGN_SECRET_KEY` and `MINISIGN_PASSWORD` as repository secrets on GitHub, and keeps a backup of
the secret key offline. The public key goes into the repository.

Order (D16): the code can be built and tested with the committed test key at any time. The release workflow gets
the signing step only together with the real public key; until the user has made the key, the release workflow
stays as it is and no release ships the updater. The first release with the updater is the first one whose
workflow signs; `--version` accepts only that release and later ones (older releases have no manifest, and the
error says so). The post-release job of 3.10 starts with the second such release.

The core crate is developed against a local `[patch]` of freemapper-core (core ADR 0006); core merges and tags
first, then freemapper switches the pin to the tag and drops the patch before its own merge (D17).

## 6. Documentation

Manual (DE/EN) chapter on installation: how updates work, the modes, rollback, what is sent. README one paragraph.
AGENTS.md: the update tools and the rule to ask before installing. Changelog DE/EN.

## 7. Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| An update breaks the program for many users at once | medium | default `notify`, `--rollback`, the post-release job |
| Windows antivirus blocks the rename or the new binary | medium | clear message, rollback, manual download stays documented |
| Users with the folder in a read-only place | low | message naming the folder |
| The signing secret leaks from GitHub | low | key rotation; the secret is only used in the release job |
| GitHub changes the latest/download redirect | low | the check fails with a clear error; fallback to the API is a later change |

## 8. Decision log

| # | Decision | Why | Cost if wrong |
|---|---|---|---|
| D1 | Install only on request by default; auto-install is opt-in (`install` mode) | an update during club training is worse than a stale version; matches "Sicheres fertig" caution | users who wanted silent updates set one switch |
| D2 | Minisign signature over a manifest, key in the binary | GitHub checksums prove nothing about origin; minisign is small, pure Rust, no dependencies | key handling falls to the user once |
| D3 | A settings file under the config folder for the mode | env vars are not usable for desktop users | the first stored user setting; a format to keep |
| D4 | Bare `.gz` executables as extra assets instead of unpacking the archives | no tar parser, smaller attack surface | release has 5 more assets |
| D5 | Fixed-name manifest via `latest/download` | no API, no rate limit, one redirect | depends on GitHub's redirect |
| D6 | Pure parts in `freemapper-update` in core, network in the app | suite rule (shared code in core) and core ADR 0007 | an extra crate and tag before freemapper can use it |
| D7 | Only the executable is updated, not the docs in the folder | docs are online and in the next full download | an AGENTS.md in the folder can lag behind |
| D8 | No install button on the browser page in the first version | the page runs a local server with a lock; an install while the page runs needs more thought | users of the page use the command or the assistant |
| D9 | Implicit check once per 24 h with the command, 3 s budget from its start, dropped if late | at most 3 s added once a day, and a fast command does not waste the day's attempt | a fast command can take up to 3 s longer once a day; a notice can arrive a day late |
| D10 | `freemapper.previous` kept for rollback | one version back covers a bad release | 20 to 40 MB more on disk |
| D11 | Implicit check only after `init`, `select`, `fetch`, `run`; not after `derive`, `export` | those two send no request today, and `--offline` exists only on the network commands | someone who only exports never sees the notice |
| D12 | A failed or dropped check counts as the day's attempt | no retry storm on a machine without network | an update notice can wait a day after a short outage |
| D13 | An older `previous` is renamed to `stale-<time>` and deleted on a later start | Windows cannot delete or overwrite a running executable, and an old MCP server may run from it | stale files pile up if they stay locked; each start tries again |
| D14 | The MCP server checks in the background and adds a note to the first tool result; it never installs on its own | most users meet freemapper through an assistant; the person decides | one extra line in one tool result per session |
| D15 | The core crate owns the host list and URL check; the app's update agent follows redirects itself | one rule for all three apps; freemapper's general agents follow any redirect | each app writes a small redirect loop |
| D16 | No release signs before the user made the key; the updater ships with the first signing release | an agent must not create or hold the signing key | the feature waits for one human step |
| D17 | Core merges and tags `freemapper-update` before freemapper merges | core ADR 0006; the apps pin tags | two merges in order |
