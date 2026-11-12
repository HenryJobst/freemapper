//! One run per project folder across processes (ADR 0090, closing the gap named in ADR 0009).
//!
//! `<project>/.freemapper-run.lock` is created exclusively by whoever starts a run, a fetch, a
//! derive, an export or an area save. The holder touches the file every 10 seconds; a lock
//! nobody touched for 60 seconds is stale and is taken over. There is no check whether a process
//! with the stored id still lives: that needs `unsafe` or a platform crate, and a crashed
//! holder costs at most one minute.
//!
//! The file holds `{pid, started, who}` and nothing else (no path, no user name). The lock path
//! is fixed; no request names it. A lock path that is a link is replaced, never followed, and a
//! project folder whose own path leads out through a link is refused (`freemapper-fs` checks).
//!
//! The claim is `OpenOptions::create_new`, not `freemapper_fs::create_new_file`: that one
//! removes an existing entry first (it is made for downloads), which would steal a live lock.

use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::errors::Localize;
use crate::text::{FluentValue, Lang, TEXTS, v};

/// The lock file in the project folder.
pub const LOCK_NAME: &str = ".freemapper-run.lock";

/// A lock nobody touched for this long is stale.
pub const STALE_AFTER: Duration = Duration::from_secs(60);

/// How often the holder touches the file.
pub const HEARTBEAT: Duration = Duration::from_secs(10);

/// How often a claim is tried again after a stale lock was removed or a rival was faster.
const ATTEMPTS: usize = 4;

/// The most the lock file may hold; a longer file is not ours.
const MAX_CONTENT_BYTES: u64 = 4096;

/// Who holds a lock; named in the message of the one who is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Who {
    /// A command in a terminal.
    Cli,
    /// The MCP server (an AI agent).
    Mcp,
    /// The area selection page.
    Page,
}

impl Who {
    fn code(self) -> &'static str {
        match self {
            Who::Cli => "cli",
            Who::Mcp => "mcp",
            Who::Page => "page",
        }
    }
}

#[derive(Debug)]
pub enum LockError {
    /// Another run holds the folder. `who` is `cli`, `mcp`, `page` or `unknown`.
    Held { who: String, since_seconds: u64 },
    /// Something other than a lock file is in the way and could not be removed.
    Stuck,
    /// The system's message.
    Io(String),
}

impl Localize for LockError {
    fn localize(&self, lang: &Lang) -> String {
        let text = |id: &str, args: &[(&str, FluentValue)]| TEXTS.text(lang, id, args);
        match self {
            LockError::Held { who, since_seconds } => {
                let started = chrono::Local::now()
                    - chrono::Duration::seconds(i64::try_from(*since_seconds).unwrap_or(i64::MAX));
                let who = match who.as_str() {
                    "cli" => text("cli-lock-who-cli", &[]),
                    "mcp" => text("cli-lock-who-mcp", &[]),
                    "page" => text("cli-lock-who-page", &[]),
                    _ => text("cli-lock-who-unknown", &[]),
                };
                text(
                    "cli-lock-held",
                    &[("time", v(started.format("%H:%M"))), ("who", v(who))],
                )
            }
            LockError::Stuck => text(
                "cli-lock-failed",
                &[("error", v(text("cli-lock-stuck", &[])))],
            ),
            LockError::Io(error) => text("cli-lock-failed", &[("error", v(error))]),
        }
    }
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.localize(&crate::text::lang()))
    }
}

impl std::error::Error for LockError {}

fn io_error(error: impl std::fmt::Display) -> LockError {
    LockError::Io(error.to_string())
}

/// What the lock file says.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Content {
    pid: u32,
    /// Unix seconds.
    started: u64,
    who: String,
}

fn unix_seconds(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// The lock of a project folder; released when dropped.
#[derive(Debug)]
pub struct FolderLock {
    path: PathBuf,
    /// The text written into the file; tells our file from a successor's.
    content: String,
    file: Option<std::fs::File>,
    stop: Option<mpsc::Sender<()>>,
    beat: Option<JoinHandle<()>>,
}

/// Releases a lock by hand (Ctrl+C), only while the file is still ours.
#[derive(Debug, Clone)]
pub struct Releaser {
    path: PathBuf,
    content: String,
}

impl Releaser {
    pub fn release(&self) {
        let ours = std::fs::symlink_metadata(&self.path).is_ok_and(|m| m.is_file())
            && read_limited(&self.path).is_some_and(|c| c == self.content);
        if ours {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

impl FolderLock {
    pub fn releaser(&self) -> Releaser {
        Releaser {
            path: self.path.clone(),
            content: self.content.clone(),
        }
    }
}

impl Drop for FolderLock {
    fn drop(&mut self) {
        drop(self.stop.take());
        if let Some(beat) = self.beat.take() {
            let _ = beat.join();
        }
        drop(self.file.take());
        self.releaser().release();
    }
}

/// The text of the regular file `path`, if it is short.
fn read_limited(path: &Path) -> Option<String> {
    let mut text = String::new();
    std::fs::File::open(path)
        .ok()?
        .take(MAX_CONTENT_BYTES)
        .read_to_string(&mut text)
        .ok()?;
    Some(text)
}

/// Takes the lock of the project folder `dir` for `who`.
pub fn acquire(dir: &Path, who: Who) -> Result<FolderLock, LockError> {
    acquire_with(dir, who, &SystemTime::now, HEARTBEAT)
}

/// Takes the lock for a command that stops with Ctrl+C: the lock file goes away then too, so
/// the next start does not wait out the minute of a stale lock.
pub fn acquire_for_command(dir: &Path, who: Who) -> Result<FolderLock, LockError> {
    let lock = acquire(dir, who)?;
    let releaser = lock.releaser();
    let _ = std::thread::Builder::new()
        .name("lock-release-on-interrupt".into())
        .spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            if runtime.block_on(tokio::signal::ctrl_c()).is_ok() {
                releaser.release();
                std::process::exit(130);
            }
        });
    Ok(lock)
}

/// Whether another run holds the lock of `dir`, without taking it: a reader that must not keep
/// a run from starting looks only. A stale lock or something that is no lock file counts as free,
/// as [`acquire`] would take its place.
pub fn check_free(dir: &Path) -> Result<(), LockError> {
    match inspect(&dir.join(LOCK_NAME), &SystemTime::now)? {
        Seen::Live { who, since_seconds } => Err(LockError::Held { who, since_seconds }),
        Seen::Gone | Seen::Junk | Seen::Stale => Ok(()),
    }
}

/// What is at the lock path.
enum Seen {
    Gone,
    /// A link, a folder or the like: not a lock file.
    Junk,
    Live {
        who: String,
        since_seconds: u64,
    },
    Stale,
}

fn inspect(path: &Path, now: &dyn Fn() -> SystemTime) -> Result<Seen, LockError> {
    let meta = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Seen::Gone),
        Err(e) => return Err(io_error(e)),
    };
    if !meta.is_file() {
        return Ok(Seen::Junk);
    }
    let modified = meta.modified().map_err(io_error)?;
    let now = now();
    // A time in the future (clocks differ a little) is fresh up to the stale limit.
    let stale = match now.duration_since(modified) {
        Ok(age) => age > STALE_AFTER,
        Err(ahead) => ahead.duration() > STALE_AFTER,
    };
    if stale {
        return Ok(Seen::Stale);
    }
    let untouched = now.duration_since(modified).unwrap_or_default();
    let content: Option<Content> =
        read_limited(path).and_then(|text| serde_json::from_str(&text).ok());
    Ok(match content {
        Some(c) if matches!(c.who.as_str(), "cli" | "mcp" | "page") => Seen::Live {
            since_seconds: unix_seconds(now).saturating_sub(c.started),
            who: c.who,
        },
        _ => Seen::Live {
            who: "unknown".into(),
            since_seconds: untouched.as_secs(),
        },
    })
}

/// [`acquire`] with the clock that judges staleness and the heartbeat interval injected.
fn acquire_with(
    dir: &Path,
    who: Who,
    now: &dyn Fn() -> SystemTime,
    heartbeat: Duration,
) -> Result<FolderLock, LockError> {
    let path = dir.join(LOCK_NAME);
    freemapper_fs::check_inside(dir, &path).map_err(|(_, e)| io_error(e))?;
    let content = serde_json::to_string(&Content {
        pid: std::process::id(),
        started: unix_seconds(now()),
        who: who.code().to_string(),
    })
    .map_err(io_error)?;
    let mut last = None;
    for _ in 0..ATTEMPTS {
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                if let Err(e) = file
                    .write_all(content.as_bytes())
                    .and_then(|()| file.flush())
                {
                    drop(file);
                    let _ = std::fs::remove_file(&path);
                    return Err(io_error(e));
                }
                return Ok(guard(path, content, file, heartbeat));
            }
            // Windows reports a file that is about to be deleted as denied; either way something
            // is there, and the next look says what.
            Err(e)
                if e.kind() == std::io::ErrorKind::AlreadyExists
                    || std::fs::symlink_metadata(&path).is_ok() => {}
            Err(e) => return Err(io_error(e)),
        }
        match inspect(&path, now)? {
            Seen::Gone => {}
            Seen::Live { who, since_seconds } => {
                return Err(LockError::Held { who, since_seconds });
            }
            Seen::Junk | Seen::Stale => {
                // Look again right before removing: the holder may have touched it meanwhile.
                if !matches!(inspect(&path, now)?, Seen::Live { .. }) {
                    freemapper_fs::remove_inside(dir, &path);
                    if std::fs::symlink_metadata(&path).is_ok() {
                        last = Some(LockError::Stuck);
                    }
                }
            }
        }
    }
    Err(last.unwrap_or(LockError::Stuck))
}

/// Starts the heartbeat for the lock that `file` holds.
fn guard(path: PathBuf, content: String, file: std::fs::File, heartbeat: Duration) -> FolderLock {
    let (stop, wait) = mpsc::channel::<()>();
    let beat = file.try_clone().ok().and_then(|touch| {
        std::thread::Builder::new()
            .name("lock-heartbeat".into())
            .spawn(move || {
                // Ends when the guard drops its sender.
                while let Err(mpsc::RecvTimeoutError::Timeout) = wait.recv_timeout(heartbeat) {
                    let _ = touch.set_modified(SystemTime::now());
                }
            })
            .ok()
    });
    FolderLock {
        path,
        content,
        file: Some(file),
        stop: Some(stop),
        beat,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lock_path(dir: &Path) -> PathBuf {
        dir.join(LOCK_NAME)
    }

    fn modified(path: &Path) -> SystemTime {
        std::fs::metadata(path).unwrap().modified().unwrap()
    }

    /// The lock file of another process (a crashed one leaves exactly this), written by hand so
    /// that no handle of ours keeps it open; Windows would keep a deleted file around then.
    fn foreign_lock(dir: &Path, who: &str) {
        let started = unix_seconds(SystemTime::now());
        std::fs::write(
            lock_path(dir),
            format!(r#"{{"pid":1,"started":{started},"who":"{who}"}}"#),
        )
        .unwrap();
    }

    fn age_it(path: &Path, seconds: u64) {
        let file = std::fs::OpenOptions::new().write(true).open(path).unwrap();
        file.set_modified(SystemTime::now() - Duration::from_secs(seconds))
            .unwrap();
    }

    #[test]
    fn second_acquire_is_refused_with_who_and_time() {
        let dir = tempfile::tempdir().unwrap();
        let _first = acquire(dir.path(), Who::Mcp).unwrap();
        let error = acquire(dir.path(), Who::Cli).unwrap_err();
        let LockError::Held { who, since_seconds } = &error else {
            panic!("{error:?}");
        };
        assert_eq!(who, "mcp");
        assert!(*since_seconds < 5);
        // Stable wording for the English tests.
        let message = error.to_string();
        assert!(
            message.starts_with("another freemapper run is using this folder (since "),
            "{message}"
        );
        assert!(message.contains("by an AI agent (MCP)"), "{message}");
        assert!(
            message.ends_with("Wait for it or stop it first."),
            "{message}"
        );
        assert!(
            !message.contains(&dir.path().display().to_string()),
            "no path in the message: {message}"
        );
    }

    #[test]
    fn check_free_looks_without_taking() {
        let dir = tempfile::tempdir().unwrap();
        check_free(dir.path()).unwrap();
        assert!(!lock_path(dir.path()).exists());
        let held = acquire(dir.path(), Who::Cli).unwrap();
        let error = check_free(dir.path()).unwrap_err();
        assert!(
            matches!(&error, LockError::Held { who, .. } if who == "cli"),
            "{error:?}"
        );
        drop(held);
        check_free(dir.path()).unwrap();
        // A lock left by a crashed run counts as free and stays where it is for `acquire`.
        foreign_lock(dir.path(), "mcp");
        assert!(check_free(dir.path()).is_err());
        age_it(&lock_path(dir.path()), STALE_AFTER.as_secs() + 5);
        check_free(dir.path()).unwrap();
        assert!(lock_path(dir.path()).exists());
    }

    #[test]
    fn the_file_holds_pid_start_and_who_only() {
        let dir = tempfile::tempdir().unwrap();
        let _lock = acquire(dir.path(), Who::Page).unwrap();
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(lock_path(dir.path())).unwrap()).unwrap();
        let keys: Vec<&String> = value.as_object().unwrap().keys().collect();
        assert_eq!(keys, ["pid", "started", "who"]);
        assert_eq!(value["who"], "page");
        assert_eq!(value["pid"], std::process::id());
    }

    #[test]
    fn stale_lock_is_taken_over() {
        let dir = tempfile::tempdir().unwrap();
        foreign_lock(dir.path(), "cli"); // a crashed holder: the file stays
        age_it(&lock_path(dir.path()), 61);
        let second = acquire(dir.path(), Who::Mcp).unwrap();
        assert!(
            std::fs::read_to_string(lock_path(dir.path()))
                .unwrap()
                .contains("\"mcp\"")
        );
        drop(second);
        assert!(!lock_path(dir.path()).exists());
    }

    #[test]
    fn staleness_follows_the_injected_clock() {
        let dir = tempfile::tempdir().unwrap();
        foreign_lock(dir.path(), "cli");
        let written = modified(&lock_path(dir.path()));
        let at = |seconds: u64| move || written + Duration::from_secs(seconds);
        let refused = acquire_with(dir.path(), Who::Mcp, &at(59), HEARTBEAT);
        assert!(matches!(refused, Err(LockError::Held { .. })));
        let taken = acquire_with(dir.path(), Who::Mcp, &at(61), HEARTBEAT);
        assert!(taken.is_ok());
    }

    #[test]
    fn a_lock_from_the_far_future_is_stale_too() {
        let dir = tempfile::tempdir().unwrap();
        foreign_lock(dir.path(), "cli");
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(lock_path(dir.path()))
            .unwrap();
        file.set_modified(SystemTime::now() + Duration::from_secs(3600))
            .unwrap();
        drop(file);
        assert!(acquire(dir.path(), Who::Mcp).is_ok());
    }

    #[test]
    fn garbled_content_still_refuses_while_fresh() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(lock_path(dir.path()), "not json").unwrap();
        match acquire(dir.path(), Who::Cli) {
            Err(LockError::Held { who, .. }) => assert_eq!(who, "unknown"),
            other => panic!("{other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn lock_that_is_a_symlink_is_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let victim = outside.path().join("victim.txt");
        std::fs::write(&victim, "precious").unwrap();
        std::os::unix::fs::symlink(&victim, lock_path(dir.path())).unwrap();
        let lock = acquire(dir.path(), Who::Cli).unwrap();
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "precious");
        let meta = std::fs::symlink_metadata(lock_path(dir.path())).unwrap();
        assert!(meta.is_file(), "the link is replaced by a regular file");
        drop(lock);
        assert_eq!(std::fs::read_to_string(&victim).unwrap(), "precious");
    }

    #[cfg(unix)]
    #[test]
    fn dangling_symlink_is_replaced_too() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let target = outside.path().join("not-there");
        std::os::unix::fs::symlink(&target, lock_path(dir.path())).unwrap();
        let _lock = acquire(dir.path(), Who::Cli).unwrap();
        assert!(!target.exists(), "nothing created through the link");
    }

    #[cfg(unix)]
    #[test]
    fn a_project_folder_that_is_a_link_still_works() {
        // The project folder itself may be a link (the user chose it); a folder inside that
        // leads away is not the lock's business. The lock sits directly in `dir`.
        let outside = tempfile::tempdir().unwrap();
        let holder = tempfile::tempdir().unwrap();
        let link = holder.path().join("project");
        std::os::unix::fs::symlink(outside.path(), &link).unwrap();
        let lock = acquire(&link, Who::Cli).unwrap();
        assert!(outside.path().join(LOCK_NAME).exists());
        drop(lock);
        assert!(!outside.path().join(LOCK_NAME).exists());
    }

    #[test]
    fn drop_removes_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let lock = acquire(dir.path(), Who::Cli).unwrap();
        assert!(lock_path(dir.path()).exists());
        drop(lock);
        assert!(!lock_path(dir.path()).exists());
        // And the folder can be locked again.
        drop(acquire(dir.path(), Who::Cli).unwrap());
    }

    #[test]
    fn panic_in_the_holder_releases_the_lock() {
        let dir = tempfile::tempdir().unwrap();
        let outcome = std::panic::catch_unwind(|| {
            let _lock = acquire(dir.path(), Who::Mcp).unwrap();
            panic!("the run failed badly");
        });
        assert!(outcome.is_err());
        assert!(!lock_path(dir.path()).exists());
        assert!(acquire(dir.path(), Who::Cli).is_ok());
    }

    #[test]
    fn heartbeat_keeps_the_lock_fresh() {
        let dir = tempfile::tempdir().unwrap();
        let lock = acquire_with(
            dir.path(),
            Who::Cli,
            &SystemTime::now,
            Duration::from_millis(50),
        )
        .unwrap();
        age_it(&lock_path(dir.path()), 45);
        std::thread::sleep(Duration::from_millis(400));
        let age = SystemTime::now()
            .duration_since(modified(&lock_path(dir.path())))
            .unwrap_or_default();
        assert!(age < Duration::from_secs(5), "touched again: {age:?}");
        drop(lock);
    }

    #[test]
    fn a_missing_folder_is_an_error_not_a_panic() {
        let dir = tempfile::tempdir().unwrap();
        let error = acquire(&dir.path().join("nope"), Who::Cli).unwrap_err();
        assert!(matches!(error, LockError::Io(_)), "{error:?}");
    }

    #[test]
    fn releasing_by_hand_removes_only_our_own_file() {
        let dir = tempfile::tempdir().unwrap();
        let lock = acquire(dir.path(), Who::Cli).unwrap();
        let releaser = lock.releaser();
        drop(lock);
        assert!(!lock_path(dir.path()).exists());
        // Someone else holds it now; the old releaser leaves it alone.
        foreign_lock(dir.path(), "mcp");
        releaser.release();
        assert!(lock_path(dir.path()).exists());
    }
}
