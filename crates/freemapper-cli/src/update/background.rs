//! The daily check after a command that uses the network anyway (spec 3.3), and in mode `install`
//! the install that follows it (spec 3.4).
//!
//! `main` starts the check on a thread before `init`, `select`, `fetch` and `run`, and finishes it
//! after the command's own output. The check gets 3 seconds from its start; a slower one is
//! dropped and recorded as failed, so it is tried again the next day.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use freemapper_update::{
    CheckRecord, Mode, Outcome, ProcessRunner, RecordResult, UpdateError, UpdateSource, Version,
    check_for_update, due, exe_path, install, load_mode, load_record, remove_stale, save_record,
};

use super::command::describe;
use super::{freemapper_app, mode_from_env, release_build, settings_dir, update_source};
use crate::text::{text, v};

/// The record of the last attempt, in the cache folder.
const RECORD: &str = "update-check.json";

/// How long after its start the check may take (spec 3.3).
const LIMIT: Duration = Duration::from_secs(3);

/// Everything the daily check depends on, so tests can set each part.
pub struct CheckContext {
    /// The cache folder of the command (`--cache-dir` or the default); the record lives there.
    pub cache_dir: Option<PathBuf>,
    /// The command runs with `--offline`.
    pub offline: bool,
    /// `CI` is set.
    pub ci: bool,
    pub mode: Mode,
    /// [`release_build`].
    pub release: bool,
    /// Unix seconds now.
    pub now: i64,
    pub source: Arc<dyn UpdateSource + Send + Sync>,
}

impl CheckContext {
    /// The context of a command with this cache folder and `--offline`, the rest from the
    /// environment, the settings and the clock.
    pub fn of_command(cache_dir: Option<PathBuf>, offline: bool) -> Self {
        let env = mode_from_env();
        let mode = match settings_dir() {
            Some(dir) => load_mode(&dir, env.as_deref()),
            None => env.and_then(|m| m.parse().ok()).unwrap_or(Mode::Notify),
        };
        Self {
            cache_dir,
            offline,
            ci: std::env::var_os("CI").is_some(),
            mode,
            release: release_build(),
            now: chrono::Utc::now().timestamp(),
            source: Arc::new(update_source()),
        }
    }
}

/// A check running on its thread.
pub struct BackgroundCheck {
    record: PathBuf,
    started: Instant,
    now: i64,
    mode: Mode,
    source: Arc<dyn UpdateSource + Send + Sync>,
    result: Receiver<Result<Outcome, UpdateError>>,
}

/// What the check has to say after the command.
#[derive(Debug, PartialEq, Eq)]
pub enum Report {
    /// A newer version exists (stderr).
    Notice(String),
    /// Mode `install` installed it (stdout, like other progress).
    Installed(String),
    /// Mode `install` could not install it (stderr).
    Warning(String),
}

impl Report {
    pub fn print(&self) {
        match self {
            Report::Installed(text) => println!("{text}"),
            Report::Notice(text) | Report::Warning(text) => eprintln!("{text}"),
        }
    }
}

/// Starts the check, or `None` when none is due: a build from source, `CI` set, `--offline`, mode
/// `off`, no cache folder, or an attempt less than 24 hours ago.
pub fn start_background_check(ctx: &CheckContext) -> Option<BackgroundCheck> {
    if !ctx.release || ctx.ci || ctx.offline || ctx.mode == Mode::Off {
        return None;
    }
    let record = ctx.cache_dir.as_ref()?.join(RECORD);
    if !due(load_record(&record).as_ref(), ctx.now) {
        return None;
    }
    let (sender, result) = std::sync::mpsc::channel();
    let source = Arc::clone(&ctx.source);
    std::thread::Builder::new()
        .name("daily check".into())
        .spawn(move || {
            // Nobody listens any more after a timeout; that is fine.
            let _ = sender.send(check_for_update(&freemapper_app(), source.as_ref(), None));
        })
        .ok()?;
    Some(BackgroundCheck {
        record,
        started: Instant::now(),
        now: ctx.now,
        mode: ctx.mode,
        source: Arc::clone(&ctx.source),
        result,
    })
}

impl BackgroundCheck {
    /// Waits for the check, at most until 3 s after its start, and records the attempt (a
    /// dropped one as failed with reason `timeout`). Only this writes the record, so a check
    /// that ends after the drop changes nothing.
    ///
    /// A newer version gives the notice. In mode `install`, after a command that succeeded, it
    /// is installed instead (`update-installing` is printed here, before the download); a failed
    /// install is one warning and never changes the exit code of the command.
    pub fn finish(self, command_ok: bool) -> Option<Report> {
        match &self.settle()? {
            Ok(outcome @ Outcome::Available { version, notes, .. }) => {
                if self.mode == Mode::Install && command_ok {
                    return Some(self.install(outcome, version, notes));
                }
                Some(Report::Notice(text(
                    "update-notice",
                    &[("new", v(version)), ("old", v(freemapper_app().current))],
                )))
            }
            // An update exists, but it needs a manual install: say so, with the link.
            Err(too_old @ UpdateError::TooOldToSelfUpdate { .. }) => {
                Some(Report::Notice(describe(too_old)))
            }
            _ => None,
        }
    }

    /// Waits for the check as [`Self::finish`] does and records the attempt. `None` when it was
    /// dropped or stopped.
    fn settle(&self) -> Option<Result<Outcome, UpdateError>> {
        let result = match self
            .result
            .recv_timeout(LIMIT.saturating_sub(self.started.elapsed()))
        {
            Ok(result) => result,
            Err(RecvTimeoutError::Timeout) => {
                self.save(RecordResult::Failed {
                    reason: "timeout".into(),
                });
                return None;
            }
            Err(RecvTimeoutError::Disconnected) => {
                self.save(RecordResult::Failed {
                    reason: "the check stopped".into(),
                });
                return None;
            }
        };
        self.save(match &result {
            Ok(Outcome::Available { version, notes, .. })
            | Err(UpdateError::TooOldToSelfUpdate { version, notes, .. }) => {
                RecordResult::Available {
                    version: *version,
                    notes: notes.clone(),
                }
            }
            Err(e) => RecordResult::Failed {
                reason: e.to_string(),
            },
            // `Current`, and any case a later core adds.
            Ok(_) => RecordResult::Current,
        });
        Some(result)
    }

    /// Mode `install`: installs `available` over the running program.
    fn install(&self, available: &Outcome, version: &Version, notes: &str) -> Report {
        let app = freemapper_app();
        println!("{}", text("update-installing", &[("version", v(version))]));
        let installed = exe_path().map_err(UpdateError::Io).and_then(|exe| {
            install(
                &app,
                self.source.as_ref(),
                &ProcessRunner::new(app.name),
                &exe,
                available,
            )
        });
        match installed {
            Ok(new) => Report::Installed(format!(
                "{}\n{}",
                text(
                    "update-done",
                    &[
                        ("old", v(app.current)),
                        ("new", v(new)),
                        ("notes", v(notes)),
                    ],
                ),
                text("update-applies-next-start", &[])
            )),
            Err(e) => Report::Warning(text(
                "update-install-failed",
                &[("new", v(version)), ("error", v(describe(&e)))],
            )),
        }
    }

    /// Records the attempt; a record that cannot be written only means the next check comes sooner.
    fn save(&self, result: RecordResult) {
        let record = CheckRecord {
            at: self.now,
            by: freemapper_app().current,
            result,
        };
        if let Some(folder) = self.record.parent() {
            let _ = std::fs::create_dir_all(folder);
        }
        let _ = save_record(&self.record, &record);
    }
}

/// The MCP server's daily check (spec 3.7): due and recorded as in 3.3, on a thread of its own.
/// It prints nothing and never installs, whatever the mode (D14). `tell` gets the note for the
/// person once the check finds a newer version, or at once when the record of an earlier check
/// knows one. A release that needs a manual install gets a note that says so, with its link.
pub fn check_for_server(ctx: &CheckContext, tell: impl FnOnce(String) + Send + 'static) {
    if let Some(check) = start_background_check(ctx) {
        let _ = std::thread::Builder::new()
            .name("update note".into())
            .spawn(move || {
                let note = match check.settle() {
                    Some(Ok(Outcome::Available { version, .. })) => Some(server_note(version)),
                    Some(Err(UpdateError::TooOldToSelfUpdate {
                        needed,
                        version,
                        notes,
                    })) => Some(text(
                        "update-mcp-note-manual",
                        &[
                            ("new", v(version)),
                            ("old", v(freemapper_app().current)),
                            ("needed", v(needed)),
                            ("notes", v(notes)),
                        ],
                    )),
                    _ => None,
                };
                if let Some(note) = note {
                    tell(note);
                }
            });
    } else if let Some((version, _)) = recorded_update(ctx) {
        tell(server_note(version));
    }
}

/// A newer version than this one, with its release notes link, that the record of an earlier
/// check holds, under the same conditions as the check itself (apart from the 24 hours). Reads
/// the record only; the area page uses it too.
pub fn recorded_update(ctx: &CheckContext) -> Option<(Version, String)> {
    if !ctx.release || ctx.ci || ctx.offline || ctx.mode == Mode::Off {
        return None;
    }
    let record = load_record(&ctx.cache_dir.as_ref()?.join(RECORD))?;
    match record.result {
        RecordResult::Available { version, notes } if version > freemapper_app().current => {
            Some((version, notes))
        }
        _ => None,
    }
}

fn server_note(version: Version) -> String {
    text(
        "update-mcp-note",
        &[("new", v(version)), ("old", v(freemapper_app().current))],
    )
}

/// Deletes the stale files an update left next to the program (spec 3.5 step 7), silently. Only
/// a release build, which is the only one that updates itself.
pub fn remove_stale_files() {
    if release_build()
        && let Ok(exe) = exe_path()
    {
        remove_stale(&exe);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Serves the committed manifest of release 99.0.0, or fails every request.
    struct FakeSource {
        serve: bool,
    }

    const MANIFEST: &[u8] = include_bytes!("../../tests/fixtures/update/manifest-99.json");
    const SIGNATURE: &[u8] = include_bytes!("../../tests/fixtures/update/manifest-99.json.minisig");

    impl UpdateSource for FakeSource {
        fn get(&self, url: &str, _max_bytes: u64) -> Result<Vec<u8>, UpdateError> {
            match (self.serve, url.ends_with(".minisig")) {
                (false, _) => Err(UpdateError::Download(format!("{url}: 503"))),
                (true, false) => Ok(MANIFEST.to_vec()),
                (true, true) => Ok(SIGNATURE.to_vec()),
            }
        }

        fn download_to(
            &self,
            url: &str,
            _size: u64,
            _file: &mut std::fs::File,
        ) -> Result<(), UpdateError> {
            panic!("the check downloads nothing: {url}")
        }
    }

    /// A due check in mode `notify` against `source`, recording in `cache`.
    fn context(cache: &std::path::Path, serve: bool) -> CheckContext {
        CheckContext {
            cache_dir: Some(cache.to_path_buf()),
            offline: false,
            ci: false,
            mode: Mode::Notify,
            release: true,
            now: 1_800_000_000,
            source: Arc::new(FakeSource { serve }),
        }
    }

    fn record_of(cache: &std::path::Path) -> Option<RecordResult> {
        load_record(&cache.join("update-check.json")).map(|record| record.result)
    }

    #[test]
    fn no_check_when_not_due() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = context(dir.path(), true);
        let recent = CheckRecord {
            at: ctx.now - 23 * 3600,
            by: super::super::freemapper_app().current,
            result: RecordResult::Current,
        };
        save_record(&dir.path().join("update-check.json"), &recent).unwrap();
        assert!(start_background_check(&ctx).is_none());
        // A day later it is due again.
        let ctx = CheckContext {
            now: ctx.now + 2 * 3600,
            ..ctx
        };
        assert!(start_background_check(&ctx).is_some());
    }

    #[test]
    fn no_check_with_offline() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = CheckContext {
            offline: true,
            ..context(dir.path(), true)
        };
        assert!(start_background_check(&ctx).is_none());
        assert_eq!(record_of(dir.path()), None);
    }

    #[test]
    fn no_check_with_ci_env() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = CheckContext {
            ci: true,
            ..context(dir.path(), true)
        };
        assert!(start_background_check(&ctx).is_none());
        assert_eq!(record_of(dir.path()), None);
    }

    #[test]
    fn no_check_in_mode_off_in_a_source_build_or_without_a_cache_folder() {
        let dir = tempfile::tempdir().unwrap();
        for ctx in [
            CheckContext {
                mode: Mode::Off,
                ..context(dir.path(), true)
            },
            CheckContext {
                release: false,
                ..context(dir.path(), true)
            },
            CheckContext {
                cache_dir: None,
                ..context(dir.path(), true)
            },
        ] {
            assert!(start_background_check(&ctx).is_none());
        }
        assert_eq!(record_of(dir.path()), None);
    }

    #[test]
    fn a_failed_check_is_recorded_and_not_shown() {
        let dir = tempfile::tempdir().unwrap();
        // The cache folder does not exist yet; the record creates it.
        let cache = dir.path().join("cache");
        let check = start_background_check(&context(&cache, false)).unwrap();
        assert_eq!(check.finish(true), None);
        let record = load_record(&cache.join("update-check.json")).unwrap();
        assert_eq!(record.at, 1_800_000_000);
        assert_eq!(record.by, super::super::freemapper_app().current);
        let RecordResult::Failed { reason } = record.result else {
            panic!("{:?}", record.result);
        };
        assert!(reason.contains("503"), "{reason}");
    }

    #[test]
    fn an_available_update_is_recorded_and_noticed() {
        let dir = tempfile::tempdir().unwrap();
        let notice = text(
            "update-notice",
            &[("new", v("99.0.0")), ("old", v(env!("CARGO_PKG_VERSION")))],
        );
        assert_eq!(
            notice,
            format!(
                "freemapper 99.0.0 is available (you have {}). Update with: freemapper update",
                env!("CARGO_PKG_VERSION")
            )
        );
        // Mode `notify` tells after any command; mode `install` tells after a failed one.
        for (mode, command_ok) in [
            (Mode::Notify, true),
            (Mode::Notify, false),
            (Mode::Install, false),
        ] {
            let ctx = CheckContext {
                mode,
                ..context(dir.path(), true)
            };
            let check = start_background_check(&ctx).unwrap();
            assert_eq!(
                check.finish(command_ok),
                Some(Report::Notice(notice.clone()))
            );
            assert_eq!(
                record_of(dir.path()),
                Some(RecordResult::Available {
                    version: "99.0.0".parse().unwrap(),
                    notes: "https://github.com/HenryJobst/freemapper/releases/tag/v99.0.0".into(),
                })
            );
            std::fs::remove_file(dir.path().join("update-check.json")).unwrap();
        }
    }
}
