# Self-update implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `freemapper update` installs the newest signed release over the running program, a daily check tells people about new versions (CLI, MCP, browser page), an opt-in mode installs on its own, and `--rollback` undoes an update.

**Architecture:** The pure parts (manifest, signature, versions, host rule, swap and rollback, settings, check record) go into a new crate `freemapper-update` in freemapper-core, with network access behind a trait the app implements (core ADR 0007). freemapper adds the HTTP source, the command, the background check, two MCP tools, a page notice, and the release-workflow steps that publish `.gz` executables and a minisign-signed manifest.

**Tech Stack:** Rust 1.99.1 (pinned), `minisign-verify` 0.3.0 (new, 2026-09-25), `flate2` 1.1.11 (already locked), `ureq` 3.4.2, `serde`/`serde_json`, `toml`, `dirs`, `freemapper-fs`, `freemapper-text`; GitHub Actions with `minisign` from the Ubuntu archive.

**Spec:** [`docs/superpowers/specs/2026-11-11-self-update-design.md`](../specs/2026-11-11-self-update-design.md), ADR [0094](../../adr/0094-self-update-from-signed-releases.md). Section numbers refer to the spec.

## Global Constraints

- Nothing unsigned or unverified is ever executed or installed; a bad signature is an error, never "no update".
- Manifest: `format = 1`, `app = "freemapper"`, `deny_unknown_fields` at the top level, unknown platforms inside `assets` ignored; manifest and signature capped at 64 KiB each.
- Hosts: `github.com`, `release-assets.githubusercontent.com`, `objects.githubusercontent.com`; https, port 443; at most 3 redirects, each checked.
- Asset names: `freemapper-<tag>-<platform>.gz`, `freemapper-update.json`, `freemapper-update.json.minisig`; platforms `linux-x86_64`, `linux-arm64`, `macos-arm64`, `macos-x86_64`, `windows-x86_64`.
- Versions are three integers compared by hand; a pre-release suffix is refused.
- Swap names: `freemapper.previous` / `freemapper.previous.exe`, stale files `freemapper.stale-<unix-seconds>` / `.exe`.
- Implicit check: only after `init`, `select`, `fetch`, `run`; never with `--offline`, with `CI` set, for `--help`/`--version`, in source builds, in mode `off`; once per 24 h counting failed attempts; 3 s budget; output one line on stderr; never on `mcp` stdout.
- Modes `notify` (default), `install`, `off` in `<config_dir>/freemapper/settings.toml` `[update] mode`; `FREEMAPPER_UPDATE` overrides. `install` never applies to the MCP server.
- Release builds carry `FREEMAPPER_DIST=github-release` (read with `option_env!` in `freemapper-cli`); without it no check and no install.
- The signing key is made by the user only. Agents never generate, read or store the secret key; tests use a committed test key pair under `tests/` that is never accepted by a release build.
- No personal data in any request; only the fixed `User-Agent`. No telemetry.
- `unsafe_code = "forbid"`; all writes through `freemapper-fs` create-new plus rename; ADR 0012 (two-week rule) for every dependency.
- User texts in German and English through the Fluent catalogs (ADR 0092); edits to `mcp.rs`, `select*.rs`, `web/` and the catalogs are agreed with the program-language session (FableWorker) before they start.
- English identifiers and docs, glossary terms ("Bundesland" in German). Changelog DE+EN per merge. Commit trailers as given by the session. unslop and humanizer over prose.
- Before each commit: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --locked`. After each push to main: watch `ci.yml` on three systems.

## Review Focus

1. The program folder is not writable (unpacked under `/usr/local/bin` or `C:\Program Files`): a message naming the folder, nothing changed, exit 1. Test in A5 (`swap_in_read_only_folder_changes_nothing`) and B2.
2. The executable is reached through a symlink (`~/.local/bin/freemapper` → unpacked folder): the target is replaced, the link stays. Test in A5 (`swap_through_symlink_replaces_the_target`, Unix only).
3. A tampered download (`.gz` or executable hash differs, or signature by another key): error naming what failed, no file in the folder changed, temp files removed. Test in A6.
4. Windows with an old MCP server still running from `freemapper.previous.exe`: the old previous becomes `stale-<time>`, the update succeeds, a later start removes the stale file once it is free. Test in A5 (`previous_in_use_becomes_stale`, Windows CI; the file held open without share-delete).
5. No network, a slow GitHub or a 500 answer during the implicit check: the command's output and exit code are unchanged, the attempt is recorded as `failed`, the next check waits 24 h. Test in B3.

---

## Stage H: the human step

### Task H1: Signing key (the user)

Not for an agent. The user runs `minisign -G -p freemapper-update.pub -s freemapper-update.key` on their own machine, stores the secret key and password as the GitHub repository secrets `MINISIGN_SECRET_KEY` (the file content) and `MINISIGN_PASSWORD`, keeps an offline backup of the secret key, and hands the public key file (only that) to the session. Stages A and B run without it, using the test key; Task B6 waits for it.

---

## Stage A: `freemapper-update` in freemapper-core

Work in `../freemapper-core` on a branch; freemapper uses it through a local `[patch]` until A7 tags it (core ADR 0006). New core ADR `0024-the-update-crate.md`; the owner table in core `CLAUDE.md` names the freemapper session (core ADR 0005).

**Done on 2026-11-11:** core v0.17.0 (main 4425241) holds `freemapper-update`; its decision is core ADR 0026 (0024 and 0025 went to the print session's crates the same day). The final review changed the API against the task texts below: `Outcome::Available` also carries `released`; `UpdateError::TooOldToSelfUpdate { needed, version, notes }`; `Outcome` and `UpdateError` are `#[non_exhaustive]` (match with a wildcard arm); the runner is `ProcessRunner::new(app.name)`; there is an install lock (`UpdateError::Busy`) and per-install temp names; `Mode` implements `FromStr` and `Display`; the check record's tag is `outcome`. Stage B uses these names.

### Task A1: Crate, versions, platform

**Files:** Create `crates/freemapper-update/{Cargo.toml,src/lib.rs,src/version.rs,src/platform.rs}`; modify the core workspace `Cargo.toml`.

**Interfaces:**
- Produces: `struct Version { major: u32, minor: u32, patch: u32 }` with `FromStr` (accepts `1.2.3` and `v1.2.3`, refuses suffixes), `Ord`, `Display` (`1.2.3`); `fn current_platform() -> Option<&'static str>` from `cfg!(target_os)` and `cfg!(target_arch)` giving the five names of Global Constraints, `None` elsewhere.

- [ ] **Step 1:** Write `version_parses_and_orders` (`"0.10.0" < "0.11.0"`, `"0.9.9" < "0.10.0"`, `"v1.0.0" == "1.0.0"`), `version_refuses_suffixes_and_junk` (`"1.0.0-rc1"`, `"1.0"`, `"1.0.0.0"`, `""`, `"01.0.0"` with its leading zero), `platform_name_is_one_of_five`. Run: FAIL (crate missing).
- [ ] **Step 2:** Implement.
- [ ] **Step 3:** `cargo test -p freemapper-update`. Expected: PASS.
- [ ] **Step 4:** Commit `feat(update): versions and platform names`.

### Task A2: Manifest

**Files:** Create `src/manifest.rs`.

**Interfaces:**
- Produces: `struct Manifest { format: u32, app: String, version: Version, released: String, notes: String, min_self_update_from: Version, assets: BTreeMap<String, Asset> }`, `struct Asset { url: String, size: u64, sha256: String, exe_sha256: String }`, `fn parse_manifest(bytes: &[u8], app: &str) -> Result<Manifest, UpdateError>`; `enum UpdateError` (in `src/error.rs`) with variants `Manifest(String)`, `Signature`, `UnknownKey`, `Url(String)`, `Download(String)`, `Hash { what: &'static str }`, `NotWritable(PathBuf)`, `VersionMismatch { expected: Version, got: String }`, `Io(std::io::Error)`, `NoAssetForPlatform(String)`, `TooOldToSelfUpdate { needed: Version }`.

- [ ] **Step 1:** Write `manifest_parses_the_spec_example` (the JSON of spec 3.1 with five assets), `manifest_refuses_unknown_top_level_key`, `manifest_ignores_unknown_platform`, `manifest_refuses_other_app` (`app = "freemapper-print"` with expected `freemapper`), `manifest_refuses_format_2`, `manifest_refuses_bad_hash_length`, `manifest_refuses_asset_url_on_foreign_host` (uses `check_url` of A4; write A4's function first if needed and note it).
- [ ] **Step 2:** Implement with `serde_json`; hashes are 64 lower-case hex characters.
- [ ] **Step 3:** Run. Expected: PASS.
- [ ] **Step 4:** Commit `feat(update): the update manifest`.

### Task A3: Signature and key list

**Files:** Create `src/signature.rs`, `tests/keys/test.pub`, `tests/keys/test.key` (made once with `minisign -G -W`, no password, for tests only), `tests/keys/other.pub`, fixtures `tests/fixtures/manifest.json` and `.minisig`, `tests/fixtures/manifest-tampered.json`.

**Interfaces:**
- Produces: `fn verify(manifest: &[u8], signature: &str, keys: &[&str]) -> Result<(), UpdateError>` where `keys` are minisign public key strings; tries each key whose id matches the signature's key id; `UnknownKey` when no id matches, `Signature` when the matching key fails.

- [ ] **Step 1:** Write `good_signature_verifies`, `tampered_manifest_fails_with_signature`, `signature_of_other_key_is_unknown_key`, `second_key_in_the_list_verifies` (rotation). Run: FAIL.
- [ ] **Step 2:** Add `minisign-verify = "=0.3.0"` (check the two-week rule again on the day) and implement.
- [ ] **Step 3:** Run. Expected: PASS. `cargo tree -p freemapper-update` shows no HTTP client.
- [ ] **Step 4:** Commit `feat(update): minisign check with a key list`.

### Task A4: Host rule, check, record and settings

**Files:** Create `src/source.rs`, `src/check.rs`, `src/record.rs`, `src/settings.rs`.

**Interfaces:**
- Produces: `pub const ALLOWED_HOSTS: [&str; 3]`; `fn check_url(url: &str) -> Result<(), UpdateError>` (https, port 443 or none, host in the list, no user info); `trait UpdateSource { fn get(&self, url: &str, max_bytes: u64) -> Result<Vec<u8>, UpdateError>; fn download_to(&self, url: &str, size: u64, file: &mut std::fs::File) -> Result<(), UpdateError>; }`; `struct App { name: &'static str, repo_url: &'static str, keys: &'static [&'static str], current: Version }`; `fn manifest_url(app: &App, version: Option<&Version>) -> String` (`{repo_url}/releases/latest/download/{name}-update.json` or `/releases/download/v{version}/…`); `enum Outcome { Current, Available { version: Version, notes: String, asset: Asset }, }`; `fn check_for_update(app: &App, source: &dyn UpdateSource, version: Option<&Version>) -> Result<Outcome, UpdateError>` (spec 3.2; with `version` it may return an older one); `struct CheckRecord { at: i64, by: Version, result: RecordResult }`, `enum RecordResult { Current, Available { version: Version, notes: String }, Failed { reason: String } }`, `fn load_record(path) -> Option<CheckRecord>`, `fn save_record(path, &CheckRecord) -> io::Result<()>`, `fn due(record: Option<&CheckRecord>, now: i64) -> bool` (24 h); `enum Mode { Notify, Install, Off }`, `fn load_mode(config_dir: &Path, env: Option<&str>) -> Mode`, `fn save_mode(config_dir: &Path, Mode) -> io::Result<()>` writing `[update] mode = "…"` and keeping other tables.

- [ ] **Step 1:** Write `check_url_accepts_the_three_hosts_only` (also refuses `http://github.com`, `https://github.com:8443`, `https://user@github.com`, `https://github.com.evil.example`), `check_finds_newer_version`, `check_equal_version_is_current`, `check_older_latest_is_current`, `check_bad_signature_is_an_error_not_current`, `check_missing_platform_is_no_asset_error`, `check_too_old_to_self_update` (min 0.12.0, current 0.10.0), `record_roundtrip_and_due_after_24_h` (due at 24 h + 1 s, not at 23 h), `failed_attempt_counts_as_attempt`, `mode_env_overrides_file`, `save_mode_keeps_other_tables` — with a `FakeSource` that serves fixture bytes from a map.
- [ ] **Step 2:** Implement; the record and settings writes use `freemapper-fs` create-new plus rename.
- [ ] **Step 3:** Run. Expected: PASS.
- [ ] **Step 4:** Commit `feat(update): check, record and mode`.

### Task A5: Swap, rollback, stale files

**Files:** Create `src/swap.rs`.

**Interfaces:**
- Produces: `fn exe_path() -> io::Result<PathBuf>` (`current_exe` then `canonicalize`); `fn swap_in(exe: &Path, new_file: &Path) -> Result<(), UpdateError>` (spec 3.5 step 6 with restore on failure); `fn rollback(exe: &Path, runner: &dyn VersionRunner) -> Result<Version, UpdateError>`; `fn remove_stale(dir: &Path)` (best effort, silent); `trait VersionRunner { fn version_of(&self, exe: &Path) -> Result<String, UpdateError>; }` with `ProcessRunner` running `<exe> --version` with a 10 s timeout and parsing `<name> <x.y.z>`; `fn previous_path(exe: &Path) -> PathBuf`.

- [ ] **Step 1:** Write `swap_keeps_old_as_previous`, `second_swap_turns_old_previous_into_stale`, `failed_second_rename_restores_the_old_exe` (inject a failure through a test hook `swap_in_with(&dyn Renamer)`), `swap_in_read_only_folder_changes_nothing` (Unix: folder mode 0o555; Windows: skip with a reason), `swap_through_symlink_replaces_the_target` (Unix), `previous_in_use_becomes_stale` (Windows: hold `previous` open with `OpenOptions` and no share-delete; Unix: runs as a plain rename test), `rollback_swaps_back_after_version_check`, `rollback_without_previous_is_an_error`, `remove_stale_ignores_locked_files`.
- [ ] **Step 2:** Implement with `std::fs::rename`; no `unsafe`, no platform crate.
- [ ] **Step 3:** Run on the three CI systems of core. Expected: PASS everywhere.
- [ ] **Step 4:** Commit `feat(update): swap, rollback and stale files`.

### Task A6: Install

**Files:** Create `src/install.rs`.

**Interfaces:**
- Consumes: A2 to A5.
- Produces: `fn install(app: &App, source: &dyn UpdateSource, runner: &dyn VersionRunner, exe: &Path, available: &Outcome) -> Result<Version, UpdateError>`: refuses `Current`; temp `.gz` in the exe's folder via `freemapper-fs` create-new; `download_to` with the manifest size; SHA-256 of the `.gz`; gunzip with `flate2` into a second temp file, refusing output larger than 512 MiB; SHA-256 of the executable; mode 0o755 on Unix; `runner.version_of(temp)` must equal the manifest version before the swap (spec 3.5 step 6); `swap_in`; temp files removed on every path.

- [ ] **Step 1:** Write `install_replaces_and_keeps_previous` (fixture `.gz` of a tiny script or the test binary itself; a `FakeRunner`), `wrong_gz_hash_changes_nothing`, `wrong_exe_hash_changes_nothing`, `size_larger_than_manifest_is_refused`, `version_mismatch_changes_nothing`, `temp_files_are_gone_after_failure`.
- [ ] **Step 2:** Implement; add `flate2 = "=1.1.11"`.
- [ ] **Step 3:** Run. Expected: PASS.
- [ ] **Step 4:** Commit `feat(update): install with verification before the swap`.

### Task A7: Core release

**Files (core):** `docs/adr/0024-the-update-crate.md`, README crate table, `CLAUDE.md` owner table, `CHANGELOG.md`/`.en.md`.

- [ ] **Step 1:** Write the ADR (network outside, test keys only in tests, owner freemapper).
- [ ] **Step 2:** Core CI green on three systems; merge; tag the next core minor release (core ADR 0006); notify print and courseplan (the crate exists, adoption is their call, keys per app or shared).
- [ ] **Step 3:** Commit `docs: the update crate`.

---

## Stage B: freemapper

Branch `self-update` from `main`. Pin the new core tag for every core crate in `Cargo.toml` (one tag for all, ADR 0067) and add `freemapper-update`; `cargo update -p` only the core crates and check the lock diff.

### Task B1: The HTTP update source

**Files:** Create `crates/freemapper-cli/src/update/{mod.rs,source.rs}`; modify `crates/freemapper-cli/Cargo.toml`, `main.rs` (module).

**Interfaces:**
- Produces: `struct HttpUpdateSource { agent: ureq::Agent }` implementing `UpdateSource`: its own agent with `max_redirects(0)`, the fixed `User-Agent` of `fetch.rs` (make that constant `pub` in the library), connect timeout 10 s; follows at most 3 redirects itself, each `Location` through `check_url`; enforces `max_bytes` and the exact `size`. `fn freemapper_app() -> App` with `keys` from `include_str!("../../../../assets/update/minisign.pub")` lines (one key per line), `current` from `CARGO_PKG_VERSION`. `fn release_build() -> bool` = `option_env!("FREEMAPPER_DIST") == Some("github-release")`. Test-only `HttpUpdateSource::stub(base: Url)` behind `#[cfg(test)]` allowing `127.0.0.1`.
- Until H1 is done, `assets/update/minisign.pub` holds the test public key with a comment line `# TEST KEY, replaced by the release key in B6`; `release_build()` is false in every build until B6 sets the variable, so the test key is never used by a shipped program.

- [ ] **Step 1:** Write `stub_serves_manifest_and_follows_one_redirect`, `redirect_to_foreign_host_is_refused`, `fourth_redirect_is_refused`, `answer_over_cap_is_refused` against a local `std::net::TcpListener` stub.
- [ ] **Step 2:** Implement.
- [ ] **Step 3:** Run `cargo test -p freemapper-cli update::`. Expected: PASS.
- [ ] **Step 4:** Commit `feat(update): http source with checked redirects`.

### Task B2: `freemapper update`

**Files:** Create `crates/freemapper-cli/src/update/command.rs`; modify `main.rs` (subcommand), both `i18n/*/freemapper.ftl` (agreed with FableWorker), `tests/cli.rs`.

**Interfaces:** `freemapper update [--check] [--yes] [--version <x.y.z>] [--rollback] [--mode notify|install|off]`. Exit codes: 0 done or current, 10 `--check` found an update, 1 error, 2 usage. Message ids (English text in the catalog; German alongside): `update-current`, `update-available` (`freemapper { $new } is available (you have { $old }).`), `update-ask` (`Install it now? [y/N]`), `update-needs-yes` (stdin not a terminal), `update-done` (`freemapper updated from { $old } to { $new }. Release notes: { $notes }`), `update-mcp-restart` (a running MCP server keeps the old version until the assistant restarts it), `update-source-build` (`This freemapper was built from source. Update it with git pull and cargo build --release --locked.`), `update-not-writable`, `update-too-old` (with the release page link), `update-rolled-back`, `update-mode-set`.

- [ ] **Step 1:** Write CLI tests: `update_in_source_build_says_how_to_update` (exit 0, `update-source-build` text), `update_check_with_stub_reports_available_exit_10` and `update_yes_installs_over_a_copied_binary` (both with a test-only env `FREEMAPPER_UPDATE_TEST_BASE` that the binary honours only under `cfg(debug_assertions)` and a `test-stub` feature, pointing at the stub; the copied binary lives in a temp folder), `update_without_yes_and_no_terminal_refuses`, `update_mode_writes_settings` (config dir via `FREEMAPPER_CONFIG_DIR` test override), `update_rollback_restores_previous`.
- [ ] **Step 2:** Implement over A4 to A6 and B1.
- [ ] **Step 3:** Run. Expected: PASS on three systems.
- [ ] **Step 4:** Commit `feat(update): the update command`.

### Task B3: The implicit check and install mode

**Files:** Create `crates/freemapper-cli/src/update/background.rs`; modify `main.rs` (start before `init`, `select`, `fetch`, `run`; print after; `remove_stale` at every start).

**Interfaces:** `fn start_background_check(ctx: &CheckContext) -> Option<BackgroundCheck>` returns `None` when not due, not a release build, mode `off`, `CI` set, or `--offline`; `BackgroundCheck::finish(self, command_ok: bool) -> Option<String>` waits at most until 3 s after start in total, writes the record (also `Failed`), returns the notice line or, in mode `install` with `command_ok`, installs and returns `update-done` plus `update-applies-next-start`. Record path `<cache>/update-check.json` (the cache folder of `--cache-dir` or the default).

- [ ] **Step 1:** Write `no_check_when_not_due`, `no_check_with_offline`, `no_check_with_ci_env`, `slow_source_is_dropped_after_3_s_and_recorded_failed` (stub that sleeps 5 s; the command's own output and exit code unchanged: Review Focus 5), `server_error_is_recorded_failed_and_not_shown`, `notice_line_on_stderr_after_command_output`, `install_mode_installs_only_after_success`, `install_mode_skips_after_failed_command`.
- [ ] **Step 2:** Implement with `std::thread` and a channel; no async.
- [ ] **Step 3:** Run. Expected: PASS.
- [ ] **Step 4:** Commit `feat(update): daily check after network commands`.

### Task B4: MCP

**Files:** Modify `crates/freemapper-cli/src/mcp.rs`, `tests/mcp.rs`, catalogs. Agree with FableWorker first.

**Interfaces:** tools `check_update` (no parameters) and `install_update { confirm: bool }` (refuses unless `true`); server start runs the background check without printing; the first tool result after a known `Available` gets `update-mcp-note` (`Note for the person: freemapper { $new } is available (this is { $old }). Ask them before calling install_update.`) once per server process; INSTRUCTIONS gain one sentence on both tools. In a source build both tools return `update-source-build`.

- [ ] **Step 1:** Write `check_update_reports_available_with_stub`, `install_update_needs_confirm_true`, `first_tool_result_carries_the_note_once`, `mcp_never_installs_in_install_mode`, `nothing_reaches_stdout_outside_the_protocol` (the existing protocol test still passes with an update known).
- [ ] **Step 2:** Implement.
- [ ] **Step 3:** Run. Expected: PASS.
- [ ] **Step 4:** Commit `feat(update): mcp tools and the update note`.

### Task B5: Browser page notice

**Files:** Modify `crates/freemapper-cli/src/select.rs` (`GET /api/update` returns `{available: null | {version, notes}}` from the record only, no network), `web/app.js`, `web/index.html`, `web/app.css`, catalogs; `tests/select.rs`. Agree with FableWorker first.

- [ ] **Step 1:** Write `api_update_reads_the_record_only` (no request leaves the process) and `api_update_null_without_record`; check the CSP test still passes.
- [ ] **Step 2:** Implement a one-line notice with the command `freemapper update`; no button (spec D8).
- [ ] **Step 3:** Run; look at the page in the browser once (both languages). Expected: PASS, notice visible with a fake record.
- [ ] **Step 4:** Commit `feat(update): notice on the area page`.

### Task B6: Release workflow (after H1)

**Files:** Modify `.github/workflows/release.yml`; replace `assets/update/minisign.pub` with the user's public key (keep the test key out of release builds: the test key moves to `crates/freemapper-cli/tests/keys/`); create `.github/scripts/update-manifest.sh`.

- [ ] **Step 1:** Build step: `env: FREEMAPPER_DIST: github-release`. Package step: also `gzip -9 -n -c <exe> > freemapper-$TAG-<platform>.gz` and its exe SHA-256 into `<platform>.exe.sha256`; upload both.
- [ ] **Step 2:** Release job: `update-manifest.sh` writes `freemapper-update.json` from the five `.gz`, their sizes and hashes (`min_self_update_from` = the first updater version, raised by hand when a release changes the update path); `sudo apt-get install -y minisign`; sign with the secrets (`printf %s "$MINISIGN_SECRET_KEY" > key; minisign -S -s key -m freemapper-update.json` with the password on stdin; `shred`/`rm` the key file in the same step); verify with `minisign -V -p assets/update/minisign.pub`; fail the job if any step fails. Upload manifest and `.minisig`.
- [ ] **Step 3:** Add a `release_build_accepts_only_the_release_key` test (the release key string in `minisign.pub` differs from the test key and `freemapper_app().keys` contains no test key when `FREEMAPPER_DIST` is set).
- [ ] **Step 4:** Add job `update-smoke` (`needs: release`, matrix of three systems, `if:` the previous release has a manifest): download the previous release's archive, unpack, `freemapper update --yes`, expect `--version` = the new tag, then `freemapper update --rollback`, expect the old one. Skipped automatically for the first updater release.
- [ ] **Step 5:** Dry run with `workflow_dispatch` on a test tag in a fork or with `--draft` release; check the manifest by `minisign -V` locally. Commit `ci(release): signed update manifest and bare executables`.

### Task B7: Documentation and merge

**Files:** `docs/manual/{de,en}.md` (installation chapter: updates, modes, rollback, what is sent, how to switch off), `README.md`/`README.en.md` (one paragraph each; README check before push), `AGENTS.md` (the two tools, "ask before install_update", the command, exit 10), `CHANGELOG.md`/`.en.md`, `docs/adr/README.md` (0094 Accepted on merge), glossary rows "Selbstaktualisierung / self-update", "Aktualisierungsmanifest / update manifest".

- [ ] **Step 1:** Write the texts; run unslop and humanizer; check every command by running it.
- [ ] **Step 2:** Final whole-branch review; merge `self-update` to `main`; watch CI; release when H1 and B6 are done (auto-release rule), then the second release exercises `update-smoke`.
- [ ] **Step 3:** Notify print, courseplan and FableWorker of the merged parts.

## Self-review

- Spec coverage: 3.1 (B6), 3.2 (A2 to A4), 3.3 (B3), 3.4 (A4, B2, B3), 3.5 (A5, A6, B2), 3.6 (B1, B2), 3.7 (B4, B5), 3.8 (A1 to A7, B1), 3.9 (A2 to A6, B1), 3.10 (A tests, B tests, B6 smoke), 5 (H1, B6), 6 (B7), D16/D17 (H1, A7, B6). Gaps: none found.
- A6 runs `--version` on the new file before the swap; the spec (3.5 steps 6 and 7) was changed to that order while writing this plan, so a broken binary never takes the place.
- Type consistency: `Version`, `Asset`, `Outcome`, `App`, `UpdateSource`, `VersionRunner`, `CheckRecord`, `Mode` defined in A1 to A5 and used by name in A6 and B1 to B4.
- Proportion: no bodies; tests named with their values.

## Size and order

| Stage | Tasks | Size | Order |
|---|---:|---|---|
| H | 1 | the user, about 15 minutes | any time before B6 |
| A | 7 | about 900 lines in core | A1 to A6 in order, A7 last |
| B | 7 | about 500 lines, 80 lines of workflow, docs | after A7's tag; B4 and B5 after agreeing with FableWorker; B6 after H1 |
| Total | 15 | | |

Recommended execution: subagent-driven. The tasks build on each other's types, the swap and install code decides whether people end up with a broken program, and a fresh reviewer per task is worth it for A5, A6 and B6 in particular.
