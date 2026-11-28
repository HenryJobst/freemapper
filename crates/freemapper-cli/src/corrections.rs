// SPDX-License-Identifier: AGPL-3.0-or-later

//! The signed list of corrections of addresses (issue #273, stage 4): a changed address reaches a
//! program without a new release.
//!
//! The list `freemapper-sources.json` and its signature lie as assets of the GitHub release
//! `sources`, a rolling release that is no "latest" one. The program fetches both the way
//! `freemapper update` fetches its manifest (the same hosts, the same redirect rule, the same
//! keys), checks the minisign signature and stores the pair in `<config>/sources/`. At every
//! start the stored pair is checked again, so a changed file in that folder counts for nothing, and
//! applied to the manifests the program shipped (`freemapper_manifest::corrections`): an address
//! is corrected only where the manifest still holds the address the correction names, and the
//! whole manifest must pass its check afterwards. A person's own adapter folder is untouched; an
//! adapter of that folder replaces the built-in one and its manifest is the person's.
//!
//! The network is asked only on request: by `freemapper sources update`, or once a day by a
//! command that uses the network anyway when the person turned the check on
//! (`sources update --mode on`). Without a stored list, or with an invalid one, the program
//! behaves as it would without this module.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::time::Duration;

use freemapper_manifest::corrections::{self as list, Applied, FILE, List, MAX_BYTES, Outcome};
use freemapper_update::{
    App, CheckRecord, RecordResult, UpdateError, UpdateSource, due, load_record, save_record,
    verify,
};

use crate::text::{text, v, warning};
use crate::update::{freemapper_app, release_build, settings_dir, update_source};

/// The tag of the release that holds the list.
const RELEASE_TAG: &str = "sources";
/// The signature of a list is a short text.
const MAX_SIGNATURE_BYTES: u64 = 64 * 1024;
/// How long the daily fetch may take after its start; a slower one is dropped and tried again the
/// next day.
const DAILY_LIMIT: Duration = Duration::from_secs(3);
/// The record of the last daily attempt.
const RECORD: &str = "check.json";

/// The folder of the stored list: `<config>/sources`.
pub fn dir() -> Option<PathBuf> {
    settings_dir().map(|dir| dir.join("sources"))
}

fn list_path(dir: &Path) -> PathBuf {
    dir.join(FILE)
}

fn signature_path(dir: &Path) -> PathBuf {
    dir.join(format!("{FILE}.minisig"))
}

/// The address of `file` of the release `sources`.
fn url_of(app: &App, file: &str) -> String {
    format!("{}/releases/download/{RELEASE_TAG}/{file}", app.repo_url)
}

/// A list read from the folder, with its signature checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stored {
    pub list: List,
    /// The bytes the signature covers.
    pub bytes: Vec<u8>,
}

/// The list stored in `dir`, when there is one: its signature checked against `keys` and its
/// content read. `Ok(None)` when no list is stored; `Err` with the reason when one is stored and
/// is not valid.
pub fn read_stored(dir: &Path, keys: &[&str]) -> Result<Option<Stored>, String> {
    let bytes = match read_limited(&list_path(dir), MAX_BYTES as u64) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(None),
        Err(reason) => return Err(reason),
    };
    let signature = read_limited(&signature_path(dir), MAX_SIGNATURE_BYTES)?
        .ok_or_else(|| "the signature is missing".to_string())?;
    let signature = String::from_utf8(signature).map_err(|_| "the signature is no text")?;
    verify(&bytes, &signature, keys).map_err(|error| error.to_string())?;
    let list = list::parse_list(&bytes).map_err(|error| error.to_string())?;
    Ok(Some(Stored { list, bytes }))
}

/// The file `path` when it is there and not larger than `max` bytes.
fn read_limited(path: &Path, max: u64) -> Result<Option<Vec<u8>>, String> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut std::io::Read::take(file, max + 1), &mut bytes)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if bytes.len() as u64 > max {
        return Err(format!("{} is larger than {max} bytes", path.display()));
    }
    Ok(Some(bytes))
}

/// The list applied to the manifests the program shipped.
pub fn apply_to_shipped(list: &List) -> Applied {
    list::apply(list, |id| {
        freemapper::adapters::shipped_manifest(id).map(str::to_string)
    })
}

/// Puts the stored list in force for this process: before anything reads a manifest. Returns the
/// warnings, which it has printed on stderr as well.
pub fn activate() -> Vec<String> {
    let Some(dir) = dir() else {
        return Vec::new();
    };
    let lines = activate_in(&dir, freemapper_app().keys, |texts| {
        freemapper::adapters::set_corrected_manifests(texts)
    });
    for line in &lines {
        eprintln!("{line}");
    }
    lines
}

/// [`activate`] for the list in `dir` with the keys `keys`; `set` puts the corrected manifests in
/// force.
fn activate_in(
    dir: &Path,
    keys: &[&str],
    set: impl FnOnce(BTreeMap<String, String>) -> bool,
) -> Vec<String> {
    let stored = match read_stored(dir, keys) {
        Ok(Some(stored)) => stored,
        Ok(None) => return Vec::new(),
        Err(reason) => {
            return vec![warning(&text(
                "cli-corrections-invalid",
                &[("reason", v(reason))],
            ))];
        }
    };
    let applied = apply_to_shipped(&stored.list);
    let lines = refusals(&applied);
    if !applied.texts.is_empty() {
        set(applied.texts);
    }
    lines
}

/// The warnings for the corrections of `applied` that were refused.
fn refusals(applied: &Applied) -> Vec<String> {
    applied
        .rows
        .iter()
        .filter_map(|row| match &row.outcome {
            Outcome::Refused(reason) => Some(warning(&text(
                "cli-corrections-refused",
                &[
                    ("adapter", v(&row.correction.adapter)),
                    ("path", v(&row.correction.path)),
                    ("reason", v(reason)),
                ],
            ))),
            _ => None,
        })
        .collect()
}

/// What a fetch did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fetched {
    /// A newer list, stored; how many of its corrections apply to this program.
    Stored { issued: String, applied: usize },
    /// The list at GitHub is the one stored.
    Current { issued: String },
    /// The list at GitHub is older than the stored one; the stored one stays.
    Older { offered: String, stored: String },
}

/// Fetches the list and its signature through `source`, checks the signature against the keys of
/// `app`, and stores the pair in `dir` unless the stored list is newer. Nothing is stored when any
/// check fails.
pub fn fetch(source: &dyn UpdateSource, app: &App, dir: &Path) -> Result<Fetched, String> {
    let url = url_of(app, FILE);
    let fail = |error: UpdateError| error.to_string();
    let bytes = source.get(&url, MAX_BYTES as u64).map_err(fail)?;
    let signature = source
        .get(&format!("{url}.minisig"), MAX_SIGNATURE_BYTES)
        .map_err(fail)?;
    let signature = String::from_utf8(signature).map_err(|_| "the signature is no text")?;
    verify(&bytes, &signature, app.keys).map_err(fail)?;
    let offered = list::parse_list(&bytes).map_err(|error| error.to_string())?;
    // A stored list that no longer verifies is no list to protect: it is replaced.
    if let Ok(Some(stored)) = read_stored(dir, app.keys) {
        if offered.issued < stored.list.issued {
            return Ok(Fetched::Older {
                offered: offered.issued,
                stored: stored.list.issued,
            });
        }
        if stored.bytes == bytes {
            return Ok(Fetched::Current {
                issued: offered.issued,
            });
        }
    }
    store(dir, &bytes, &signature).map_err(|error| error.to_string())?;
    let applied = apply_to_shipped(&offered)
        .rows
        .iter()
        .filter(|row| matches!(row.outcome, Outcome::Applied { .. }))
        .count();
    Ok(Fetched::Stored {
        issued: offered.issued,
        applied,
    })
}

/// Writes the pair: each file whole or not at all.
fn store(dir: &Path, bytes: &[u8], signature: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    write_whole(&signature_path(dir), signature.as_bytes())?;
    write_whole(&list_path(dir), bytes)
}

fn write_whole(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let temporary = path.with_extension("new");
    std::fs::write(&temporary, bytes)?;
    std::fs::rename(&temporary, path)
}

/// Removes the stored list; `false` when none was stored.
pub fn remove(dir: &Path) -> std::io::Result<bool> {
    let mut removed = false;
    for path in [list_path(dir), signature_path(dir), dir.join(RECORD)] {
        match std::fs::remove_file(&path) {
            Ok(()) => removed = true,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(removed)
}

/// The words of `[sources] corrections` in `settings.toml`.
const SETTING_ON: &str = "on";
const SETTING_OFF: &str = "off";

fn settings_file() -> Option<PathBuf> {
    settings_dir().map(|dir| dir.join("settings.toml"))
}

/// Whether the person turned the daily check on (`[sources] corrections = "on"`); the default is off.
pub fn daily_on() -> bool {
    settings_file().is_some_and(|file| daily_on_in(&file))
}

fn daily_on_in(file: &Path) -> bool {
    std::fs::read_to_string(file)
        .ok()
        .and_then(|text| text.parse::<toml::Table>().ok())
        .and_then(|table| {
            table
                .get("sources")?
                .as_table()?
                .get("corrections")?
                .as_str()
                .map(|value| value == SETTING_ON)
        })
        .unwrap_or(false)
}

/// Stores `[sources] corrections` in `settings.toml`, keeping the other settings (comments are
/// lost, as with the update mode). A file that is no TOML stays as it is and is an error.
pub fn set_daily(file: &Path, on: bool) -> std::io::Result<()> {
    let mut table = match std::fs::read_to_string(file) {
        Ok(text) => text
            .parse::<toml::Table>()
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => toml::Table::new(),
        Err(error) => return Err(error),
    };
    let sources = table
        .entry("sources")
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));
    if !sources.is_table() {
        *sources = toml::Value::Table(toml::Table::new());
    }
    if let Some(sources) = sources.as_table_mut() {
        sources.insert(
            "corrections".into(),
            toml::Value::String(if on { SETTING_ON } else { SETTING_OFF }.into()),
        );
    }
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    write_whole(file, table.to_string().as_bytes())
}

/// Everything the daily fetch depends on, so that tests can set each part.
pub struct Daily {
    pub dir: PathBuf,
    pub app: App,
    pub source: Arc<dyn UpdateSource + Send + Sync>,
    /// The command runs with `--offline`.
    pub offline: bool,
    /// `CI` is set.
    pub ci: bool,
    /// The person turned the check on.
    pub on: bool,
    /// [`release_build`].
    pub release: bool,
    /// Unix seconds now.
    pub now: i64,
}

/// The daily fetch of a command that uses the network anyway: once a day, only when turned on,
/// never offline or in CI, and for at most [`DAILY_LIMIT`]. A failure is recorded, said nothing
/// about, and tried again the next day. Returns the line to tell the person when a newer list
/// was stored.
pub fn refresh_daily(daily: Daily) -> Option<String> {
    if !daily.on || daily.offline || daily.ci || !daily.release {
        return None;
    }
    let record_path = daily.dir.join(RECORD);
    if !due(load_record(&record_path).as_ref(), daily.now) {
        return None;
    }
    let (sender, receiver) = channel();
    let (source, app, dir) = (
        Arc::clone(&daily.source),
        daily.app.clone(),
        daily.dir.clone(),
    );
    std::thread::spawn(move || {
        let _ = sender.send(fetch(source.as_ref(), &app, &dir));
    });
    let result = match receiver.recv_timeout(DAILY_LIMIT) {
        Ok(result) => result,
        Err(RecvTimeoutError::Timeout) => Err("it took longer than the limit".to_string()),
        Err(RecvTimeoutError::Disconnected) => Err("the fetch stopped".to_string()),
    };
    let record = CheckRecord {
        at: daily.now,
        by: daily.app.current.clone(),
        result: match &result {
            Ok(_) => RecordResult::Current,
            Err(reason) => RecordResult::Failed {
                reason: reason.clone(),
            },
        },
    };
    let _ = std::fs::create_dir_all(&daily.dir);
    let _ = save_record(&record_path, &record);
    match result {
        Ok(Fetched::Stored { issued, applied }) => Some(text(
            "cli-corrections-daily-stored",
            &[("issued", v(issued)), ("applied", crate::text::n(applied))],
        )),
        _ => None,
    }
}

/// [`refresh_daily`] for a command: its `--offline`, the environment, the settings and the clock.
pub fn refresh_daily_for_command(offline: bool) {
    let Some(dir) = dir() else { return };
    let line = refresh_daily(Daily {
        dir,
        app: freemapper_app(),
        source: Arc::new(update_source()),
        offline,
        ci: std::env::var_os("CI").is_some(),
        on: daily_on(),
        release: release_build(),
        now: chrono::Utc::now().timestamp(),
    });
    if let Some(line) = line {
        eprintln!("{line}");
    }
}

/// `freemapper sources update`: with `mode` only the setting of the daily check, with `remove`
/// only the stored list gone, else the list fetched now.
pub fn update(mode: Option<bool>, remove_list: bool) -> anyhow::Result<()> {
    let config =
        settings_dir().ok_or_else(|| anyhow::anyhow!(text("update-no-config-dir", &[])))?;
    let dir = config.join("sources");
    if let Some(on) = mode {
        let file = config.join("settings.toml");
        set_daily(&file, on).map_err(|error| {
            anyhow::anyhow!(text(
                "update-mode-failed",
                &[("path", v(file.display())), ("error", v(error))]
            ))
        })?;
        println!(
            "{}",
            text(
                "cli-sources-update-mode-set",
                &[
                    ("mode", v(if on { SETTING_ON } else { SETTING_OFF })),
                    ("path", v(file.display())),
                ]
            )
        );
        return Ok(());
    }
    if remove_list {
        let message = if remove(&dir)? {
            "cli-sources-update-removed"
        } else {
            "cli-sources-update-none"
        };
        println!("{}", text(message, &[]));
        return Ok(());
    }
    if !release_build() {
        println!("{}", text("cli-sources-update-source-build", &[]));
        return Ok(());
    }
    let fetched = fetch(&update_source(), &freemapper_app(), &dir).map_err(|reason| {
        anyhow::anyhow!(text("cli-sources-update-failed", &[("reason", v(reason))]))
    })?;
    let line = match fetched {
        Fetched::Stored { issued, applied } => text(
            "cli-sources-update-stored",
            &[("issued", v(issued)), ("applied", crate::text::n(applied))],
        ),
        Fetched::Current { issued } => text("cli-sources-update-current", &[("issued", v(issued))]),
        Fetched::Older { offered, stored } => text(
            "cli-sources-update-older",
            &[("offered", v(offered)), ("stored", v(stored))],
        ),
    };
    println!("{line}");
    Ok(())
}

/// The word that tells what became of a correction; the same in every language, as the status of
/// `adapters probe` is.
fn code_of(outcome: &Outcome) -> &'static str {
    match outcome {
        Outcome::Applied { .. } => "applied",
        Outcome::AlreadyCurrent => "already-current",
        Outcome::Stale => "stale",
        Outcome::UnknownAdapter => "unknown-adapter",
        Outcome::UnknownPath => "unknown-path",
        Outcome::Refused(_) => "refused",
    }
}

/// `freemapper sources corrections [--json]`.
pub fn show(json: bool) -> anyhow::Result<()> {
    let dir = dir().ok_or_else(|| anyhow::anyhow!(text("update-no-config-dir", &[])))?;
    show_in(&dir, freemapper_app().keys, json)
}

fn show_in(dir: &Path, keys: &[&str], json: bool) -> anyhow::Result<()> {
    let stored = match read_stored(dir, keys) {
        Ok(Some(stored)) => stored,
        Ok(None) => {
            if json {
                println!("{}", serde_json::json!({"issued": null, "corrections": []}));
            } else {
                println!("{}", text("cli-sources-update-none", &[]));
            }
            return Ok(());
        }
        Err(reason) => anyhow::bail!(text("cli-corrections-invalid", &[("reason", v(reason))])),
    };
    let applied = apply_to_shipped(&stored.list);
    if json {
        let rows: Vec<_> = applied
            .rows
            .iter()
            .map(|row| {
                serde_json::json!({
                    "adapter": row.correction.adapter,
                    "path": row.correction.path,
                    "from": row.correction.from,
                    "to": row.correction.to,
                    "add_host": row.correction.add_host,
                    "note": row.correction.note,
                    "outcome": code_of(&row.outcome),
                    "detail": match &row.outcome {
                        Outcome::Refused(reason) => Some(reason.clone()),
                        Outcome::Applied { host_added } => host_added.clone(),
                        _ => None,
                    },
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::json!({"issued": stored.list.issued, "corrections": rows})
        );
        return Ok(());
    }
    println!(
        "{}",
        text(
            "cli-sources-corrections-head",
            &[
                ("issued", v(&stored.list.issued)),
                ("count", crate::text::n(applied.rows.len())),
            ]
        )
    );
    for row in &applied.rows {
        let correction = &row.correction;
        println!(
            "{:<15} {}.{}: {} -> {}",
            code_of(&row.outcome),
            correction.adapter,
            correction.path,
            correction.from,
            correction.to
        );
        if let Outcome::Refused(reason) = &row.outcome {
            println!("  {reason}");
        }
        if !correction.note.is_empty() {
            println!("  {}", correction.note);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    const LIST: &[u8] = include_bytes!("../tests/fixtures/sources/list-2026-11-27.json");
    const SIGNATURE: &str = include_str!("../tests/fixtures/sources/list-2026-11-27.json.minisig");
    const WRONG_KEY: &str =
        include_str!("../tests/fixtures/sources/list-2026-11-27.json.wrong-key.minisig");
    const OLDER: &[u8] = include_bytes!("../tests/fixtures/sources/list-2026-11-20.json");
    const OLDER_SIGNATURE: &str =
        include_str!("../tests/fixtures/sources/list-2026-11-20.json.minisig");

    const OLD_ADDRESS: &str = "https://geodienste.hamburg.de/WFS_HH_ALKIS_vereinfacht";
    const NEW_ADDRESS: &str = "https://geodienste.hamburg.de/WFS_HH_ALKIS_vereinfacht_v2";

    /// The GitHub release as a set of files; every request is noted.
    struct Fake {
        files: Mutex<HashMap<String, Vec<u8>>>,
        asked: Mutex<Vec<String>>,
    }

    impl Fake {
        fn with(list: &[u8], signature: &str) -> Self {
            let app = freemapper_app();
            let fake = Self {
                files: Mutex::new(HashMap::new()),
                asked: Mutex::new(Vec::new()),
            };
            fake.put(list, signature, &app);
            fake
        }

        fn empty() -> Self {
            Self {
                files: Mutex::new(HashMap::new()),
                asked: Mutex::new(Vec::new()),
            }
        }

        fn put(&self, list: &[u8], signature: &str, app: &App) {
            let mut files = self.files.lock().unwrap();
            files.insert(url_of(app, FILE), list.to_vec());
            files.insert(
                url_of(app, &format!("{FILE}.minisig")),
                signature.as_bytes().to_vec(),
            );
        }

        fn requests(&self) -> usize {
            self.asked.lock().unwrap().len()
        }
    }

    impl UpdateSource for Fake {
        fn get(&self, url: &str, max_bytes: u64) -> Result<Vec<u8>, UpdateError> {
            self.asked.lock().unwrap().push(url.to_string());
            match self.files.lock().unwrap().get(url) {
                Some(bytes) if bytes.len() as u64 <= max_bytes => Ok(bytes.clone()),
                Some(_) => Err(UpdateError::Download(format!("{url}: too large"))),
                None => Err(UpdateError::Download(format!("{url}: HTTP 404"))),
            }
        }

        fn download_to(
            &self,
            _url: &str,
            _size: u64,
            _file: &mut std::fs::File,
        ) -> Result<(), UpdateError> {
            unreachable!("the list is fetched into memory")
        }
    }

    fn stored_in(dir: &Path, list: &[u8], signature: &str) {
        store(dir, list, signature).unwrap();
    }

    #[test]
    fn the_list_lies_in_a_release_that_is_no_latest_one() {
        let app = freemapper_app();
        assert_eq!(
            url_of(&app, FILE),
            "https://github.com/HenryJobst/freemapper/releases/download/sources/freemapper-sources.json"
        );
    }

    #[test]
    fn a_stored_list_counts_only_with_its_signature() {
        let app = freemapper_app();
        let dir = tempfile::tempdir().unwrap();
        // Nothing stored: nothing to read, no complaint.
        assert_eq!(read_stored(dir.path(), app.keys), Ok(None));
        stored_in(dir.path(), LIST, SIGNATURE);
        let stored = read_stored(dir.path(), app.keys).unwrap().unwrap();
        assert_eq!(stored.list.issued, "2026-11-27");
        assert_eq!(stored.bytes, LIST);
        // A changed list no longer matches its signature.
        let tampered = String::from_utf8_lossy(LIST).replace("_v2", "_evil");
        stored_in(dir.path(), tampered.as_bytes(), SIGNATURE);
        assert!(read_stored(dir.path(), app.keys).is_err());
        // A signature of a key the program does not know.
        stored_in(dir.path(), LIST, WRONG_KEY);
        assert!(read_stored(dir.path(), app.keys).is_err());
        // A list without its signature.
        stored_in(dir.path(), LIST, SIGNATURE);
        std::fs::remove_file(signature_path(dir.path())).unwrap();
        let missing = read_stored(dir.path(), app.keys).unwrap_err();
        assert!(missing.contains("signature"), "{missing}");
        // A list with a valid signature that is no list of this program's format.
        let wrong_format = String::from_utf8_lossy(LIST).replace("\"format\": 1", "\"format\": 7");
        assert!(list::parse_list(wrong_format.as_bytes()).is_err());
    }

    #[test]
    fn a_list_larger_than_the_limit_is_not_read() {
        let app = freemapper_app();
        let dir = tempfile::tempdir().unwrap();
        stored_in(dir.path(), &vec![b' '; MAX_BYTES + 1], SIGNATURE);
        let reason = read_stored(dir.path(), app.keys).unwrap_err();
        assert!(reason.contains("larger"), "{reason}");
        // Exactly at the limit it is read (and fails its signature, not its size).
        stored_in(dir.path(), &vec![b' '; MAX_BYTES], SIGNATURE);
        let reason = read_stored(dir.path(), app.keys).unwrap_err();
        assert!(!reason.contains("larger"), "{reason}");
    }

    #[test]
    fn activation_puts_the_corrected_manifests_in_force_and_says_what_it_leaves_out() {
        let app = freemapper_app();
        let dir = tempfile::tempdir().unwrap();
        let mut set_with = None;
        // No list: nothing is set.
        let lines = activate_in(dir.path(), app.keys, |texts| {
            set_with = Some(texts);
            true
        });
        assert!(lines.is_empty() && set_with.is_none());
        stored_in(dir.path(), LIST, SIGNATURE);
        let lines = activate_in(dir.path(), app.keys, |texts| {
            set_with = Some(texts);
            true
        });
        assert!(lines.is_empty(), "{lines:?}");
        let texts = set_with.take().unwrap();
        assert_eq!(texts.keys().collect::<Vec<_>>(), ["hamburg"]);
        let shipped = freemapper::adapters::shipped_manifest("hamburg").unwrap();
        assert_eq!(texts["hamburg"], shipped.replace(OLD_ADDRESS, NEW_ADDRESS));
        // A list that does not verify changes nothing and is named.
        stored_in(dir.path(), LIST, WRONG_KEY);
        let lines = activate_in(dir.path(), app.keys, |texts| {
            set_with = Some(texts);
            true
        });
        assert!(set_with.is_none());
        assert_eq!(lines.len(), 1);
        assert!(
            lines[0].contains("left out") || lines[0].contains("nicht"),
            "{lines:?}"
        );
    }

    #[test]
    fn only_a_refused_correction_is_a_warning() {
        let correction = |adapter: &str| list::Correction {
            adapter: adapter.into(),
            path: "alkis.url".into(),
            from: "https://a.example.org/a".into(),
            to: "https://b.example.org/b".into(),
            add_host: false,
            note: String::new(),
        };
        let row = |adapter: &str, outcome| list::Row {
            correction: correction(adapter),
            outcome,
        };
        let applied = Applied {
            texts: BTreeMap::new(),
            rows: vec![
                row("hamburg", Outcome::Applied { host_added: None }),
                row("berlin", Outcome::Refused("no host b.example.org".into())),
                row("bremen", Outcome::Stale),
                row("saarland", Outcome::UnknownPath),
            ],
        };
        let lines = refusals(&applied);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(lines[0].contains("berlin"), "{lines:?}");
        assert!(lines[0].contains("alkis.url"), "{lines:?}");
        assert!(lines[0].contains("no host b.example.org"), "{lines:?}");
    }

    #[test]
    fn the_stored_fixture_applies_one_correction_and_names_the_unknown_adapter() {
        let app = freemapper_app();
        let dir = tempfile::tempdir().unwrap();
        stored_in(dir.path(), LIST, SIGNATURE);
        let stored = read_stored(dir.path(), app.keys).unwrap().unwrap();
        let applied = apply_to_shipped(&stored.list);
        assert_eq!(applied.rows.len(), 2);
        assert_eq!(
            applied.rows[0].outcome,
            Outcome::Applied { host_added: None }
        );
        assert_eq!(applied.rows[1].outcome, Outcome::UnknownAdapter);
    }

    #[test]
    fn a_fetch_stores_a_checked_list_and_only_that() {
        let app = freemapper_app();
        let dir = tempfile::tempdir().unwrap();
        let source = Fake::with(LIST, SIGNATURE);
        let fetched = fetch(&source, &app, dir.path()).unwrap();
        assert_eq!(
            fetched,
            Fetched::Stored {
                issued: "2026-11-27".into(),
                applied: 1
            }
        );
        assert_eq!(std::fs::read(list_path(dir.path())).unwrap(), LIST);
        assert_eq!(
            std::fs::read_to_string(signature_path(dir.path())).unwrap(),
            SIGNATURE
        );
        // The same list again: nothing new.
        assert_eq!(
            fetch(&source, &app, dir.path()).unwrap(),
            Fetched::Current {
                issued: "2026-11-27".into()
            }
        );
    }

    #[test]
    fn a_fetch_that_fails_a_check_stores_nothing() {
        let app = freemapper_app();
        for (list, signature, why) in [
            (LIST.to_vec(), WRONG_KEY, "a key the program does not know"),
            (
                String::from_utf8_lossy(LIST)
                    .replace("_v2", "_evil")
                    .into_bytes(),
                SIGNATURE,
                "a changed list",
            ),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let source = Fake::with(&list, signature);
            assert!(fetch(&source, &app, dir.path()).is_err(), "{why}");
            assert!(!list_path(dir.path()).exists(), "{why}");
            assert!(!signature_path(dir.path()).exists(), "{why}");
        }
        // Nothing at GitHub.
        let dir = tempfile::tempdir().unwrap();
        let reason = fetch(&Fake::empty(), &app, dir.path()).unwrap_err();
        assert!(reason.contains("404"), "{reason}");
        assert!(!list_path(dir.path()).exists());
    }

    #[test]
    fn an_older_list_never_replaces_a_newer_one() {
        let app = freemapper_app();
        let dir = tempfile::tempdir().unwrap();
        stored_in(dir.path(), LIST, SIGNATURE);
        let source = Fake::with(OLDER, OLDER_SIGNATURE);
        assert_eq!(
            fetch(&source, &app, dir.path()).unwrap(),
            Fetched::Older {
                offered: "2026-11-20".into(),
                stored: "2026-11-27".into()
            }
        );
        assert_eq!(std::fs::read(list_path(dir.path())).unwrap(), LIST);
        // A newer one replaces an older one.
        let other = tempfile::tempdir().unwrap();
        stored_in(other.path(), OLDER, OLDER_SIGNATURE);
        let source = Fake::with(LIST, SIGNATURE);
        assert!(matches!(
            fetch(&source, &app, other.path()).unwrap(),
            Fetched::Stored { .. }
        ));
        assert_eq!(std::fs::read(list_path(other.path())).unwrap(), LIST);
    }

    #[test]
    fn a_stored_pair_that_no_longer_verifies_is_replaced_by_a_fetch() {
        let app = freemapper_app();
        let dir = tempfile::tempdir().unwrap();
        stored_in(dir.path(), LIST, WRONG_KEY);
        let source = Fake::with(OLDER, OLDER_SIGNATURE);
        assert!(matches!(
            fetch(&source, &app, dir.path()).unwrap(),
            Fetched::Stored { .. }
        ));
        assert_eq!(std::fs::read(list_path(dir.path())).unwrap(), OLDER);
    }

    #[test]
    fn removing_the_list_says_whether_there_was_one() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!remove(dir.path()).unwrap());
        stored_in(dir.path(), LIST, SIGNATURE);
        assert!(remove(dir.path()).unwrap());
        assert!(!list_path(dir.path()).exists() && !signature_path(dir.path()).exists());
        assert!(!remove(dir.path()).unwrap());
    }

    #[test]
    fn the_daily_check_is_a_setting_that_keeps_the_others() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("settings.toml");
        assert!(!daily_on_in(&file));
        std::fs::write(&file, "[update]\nmode = \"off\"\n").unwrap();
        set_daily(&file, true).unwrap();
        assert!(daily_on_in(&file));
        let kept: toml::Table = std::fs::read_to_string(&file).unwrap().parse().unwrap();
        assert_eq!(kept["update"]["mode"].as_str(), Some("off"));
        set_daily(&file, false).unwrap();
        assert!(!daily_on_in(&file));
        // A settings file that is no TOML stays as it is.
        std::fs::write(&file, "not [ toml").unwrap();
        assert!(set_daily(&file, true).is_err());
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "not [ toml");
        // Another word than `on` is off.
        std::fs::write(&file, "[sources]\ncorrections = \"yes\"\n").unwrap();
        assert!(!daily_on_in(&file));
    }

    fn daily(dir: &Path, source: &Arc<Fake>) -> Daily {
        Daily {
            dir: dir.to_path_buf(),
            app: freemapper_app(),
            source: Arc::clone(source) as Arc<dyn UpdateSource + Send + Sync>,
            offline: false,
            ci: false,
            on: true,
            release: true,
            now: 1_000_000,
        }
    }

    #[test]
    fn the_daily_fetch_asks_once_a_day_and_only_when_it_may() {
        let dir = tempfile::tempdir().unwrap();
        let source = Arc::new(Fake::with(LIST, SIGNATURE));
        // Each of these keeps it from asking at all.
        for (name, change) in [
            ("off", (|d: &mut Daily| d.on = false) as fn(&mut Daily)),
            ("offline", |d| d.offline = true),
            ("CI", |d| d.ci = true),
            ("a build from source", |d| d.release = false),
        ] {
            let mut request = daily(dir.path(), &source);
            change(&mut request);
            assert_eq!(refresh_daily(request), None, "{name}");
            assert_eq!(source.requests(), 0, "{name}");
        }
        // Turned on and due: it fetches and tells.
        let line = refresh_daily(daily(dir.path(), &source)).unwrap();
        assert!(line.contains("2026-11-27"), "{line}");
        assert_eq!(source.requests(), 2);
        // The same day again: not due, no request.
        assert_eq!(refresh_daily(daily(dir.path(), &source)), None);
        assert_eq!(source.requests(), 2);
        // A day later it asks again and, finding the same list, says nothing.
        let mut later = daily(dir.path(), &source);
        later.now += 24 * 60 * 60 + 1;
        assert_eq!(refresh_daily(later), None);
        assert_eq!(source.requests(), 4);
    }

    #[test]
    fn a_failed_daily_fetch_is_recorded_and_not_repeated_the_same_day() {
        let dir = tempfile::tempdir().unwrap();
        let source = Arc::new(Fake::empty());
        assert_eq!(refresh_daily(daily(dir.path(), &source)), None);
        let asked = source.requests();
        assert!(asked >= 1);
        let record = load_record(&dir.path().join(RECORD)).unwrap();
        assert!(matches!(record.result, RecordResult::Failed { .. }));
        assert_eq!(refresh_daily(daily(dir.path(), &source)), None);
        assert_eq!(source.requests(), asked);
        assert!(!list_path(dir.path()).exists());
    }
}
