//! Files the Bayerische Vermessungsverwaltung publishes under a fixed address and replaces now
//! and then (height tiles, LoD2 tiles, the land use of a Kreis): the cached copy carries the day
//! the server last changed the file, so one `HEAD` tells whether it is still the current one.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::fetch::{FetchError, FetchMode, Fetcher, write_cache_file};
use crate::note::{Arg, Note, note};

/// How long a 404 is believed, and a file of a server that sends no `Last-Modified`.
pub const RECHECK_AFTER: Duration = Duration::from_secs(30 * 24 * 60 * 60);

/// The day in the name of a file whose server sends no `Last-Modified`.
const UNDATED: &str = "00000000";

/// A file the agency publishes under a fixed address and replaces now and then: where its
/// copies are cached (`<dir>/<stem>-<YYYYMMDD>.<ext>`) and where it is fetched from.
#[derive(Debug, Clone, Copy)]
pub struct Published<'a> {
    pub dir: &'a Path,
    pub stem: &'a str,
    pub ext: &'a str,
    pub url: &'a str,
}

/// What is to be done about a published file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Current {
    /// The cached file is the one to read.
    Cached(PathBuf),
    /// Download into `path`; once that is written, `older` can go.
    Fetch { path: PathBuf, older: Vec<PathBuf> },
    /// The server does not have the file.
    Missing,
}

/// The cached copies of `stem`, each with the day in its name, the newest last.
fn copies(dir: &Path, stem: &str, ext: &str) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut copies: Vec<(String, PathBuf)> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let day = name
                .strip_prefix(stem)?
                .strip_prefix('-')?
                .strip_suffix(ext)?
                .strip_suffix('.')?;
            (day.len() == 8 && day.bytes().all(|b| b.is_ascii_digit()))
                .then(|| (day.to_string(), entry.path()))
        })
        .collect();
    copies.sort();
    copies
}

/// The newest cached file of `stem`, without asking.
pub fn cached(dir: &Path, stem: &str, ext: &str) -> Option<PathBuf> {
    copies(dir, stem, ext).pop().map(|(_, path)| path)
}

/// Where the note is kept that the server answered 404 for `stem`.
pub fn missing_note(dir: &Path, stem: &str) -> PathBuf {
    dir.join(format!("{stem}.missing"))
}

/// Whether the cached file is still the one the server has.
///
/// Offline nothing is asked: the newest cached file is used, and a file the server did not
/// have stays missing. Online one `HEAD` gives the day the file was last changed. A 404 is
/// believed for [`RECHECK_AFTER`]; so is a cached file of a server that names no day. When
/// the check itself fails, the newest cached file is used and a warning says so; without one
/// the failure is the error.
pub fn current(
    fetcher: &dyn Fetcher,
    mode: FetchMode,
    file: &Published<'_>,
    progress: &mut dyn FnMut(Note),
) -> Result<Current, FetchError> {
    let mut copies = copies(file.dir, file.stem, file.ext);
    let newest = copies.last().cloned();
    let note = missing_note(file.dir, file.stem);
    let fresh = |path: &Path| crate::clock::age_of(path).is_some_and(|age| age < RECHECK_AFTER);
    if mode == FetchMode::Offline {
        return match newest {
            Some((_, path)) => Ok(Current::Cached(path)),
            None if note.exists() => Ok(Current::Missing),
            None => Err(FetchError::Http {
                url: file.url.to_string(),
                message: format!(
                    "offline and not cached: {}; run `freemapper fetch` online",
                    file.url
                ),
            }),
        };
    }
    match &newest {
        None if fresh(&note) => return Ok(Current::Missing),
        Some((day, path)) if day == UNDATED && fresh(path) => {
            return Ok(Current::Cached(path.clone()));
        }
        _ => {}
    }
    match fetcher.last_modified(file.url) {
        Ok(changed) => {
            if note.exists() {
                let _ = std::fs::remove_file(&note);
            }
            let day = changed.map_or_else(
                || UNDATED.to_string(),
                |date| date.format("%Y%m%d").to_string(),
            );
            let path = file.dir.join(format!("{}-{day}.{}", file.stem, file.ext));
            if changed.is_some() && path.exists() {
                return Ok(Current::Cached(path));
            }
            copies.retain(|(_, copy)| *copy != path);
            Ok(Current::Fetch {
                path,
                older: copies.into_iter().map(|(_, copy)| copy).collect(),
            })
        }
        Err(FetchError::NotFound(_)) => match newest {
            // The file left the server; what is cached is still the best there is.
            Some((_, path)) => Ok(Current::Cached(path)),
            None => {
                write_cache_file(&note, b"")?;
                Ok(Current::Missing)
            }
        },
        Err(e) => match newest {
            Some((_, path)) => {
                progress(note!(
                    "warning-file-check-failed",
                    file = file.url,
                    reason = Arg::error(e)
                ));
                Ok(Current::Cached(path))
            }
            None => Err(e),
        },
    }
}

/// Removes the older copies a [`Current::Fetch`] named, once the new file is written.
pub fn drop_older(older: &[PathBuf]) {
    for path in older {
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;
    use crate::fetch::test_support::MapFetcher;

    const URL: &str = "https://download1.bayernwolke.de/a/tn/lkr/tn_09565.zip";

    fn day(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    /// A server that has the file, last changed on `changed` (or without that header).
    fn server(changed: Option<NaiveDate>) -> MapFetcher {
        let fetcher = MapFetcher::default();
        fetcher.insert(URL, b"zip".to_vec());
        if let Some(changed) = changed {
            fetcher.set_modified(URL, changed);
        }
        fetcher
    }

    /// What `current` says for the land use file of Schwabach in `dir`, with its notes.
    fn ask(
        fetcher: &MapFetcher,
        mode: FetchMode,
        dir: &Path,
    ) -> (Result<Current, FetchError>, Vec<Note>) {
        let mut notes = Vec::new();
        let file = Published {
            dir,
            stem: "tn_09565",
            ext: "zip",
            url: URL,
        };
        let found = current(fetcher, mode, &file, &mut |note| notes.push(note));
        (found, notes)
    }

    fn in_days(days: i64) -> chrono::NaiveDateTime {
        chrono::Utc::now().naive_utc() + chrono::Duration::days(days)
    }

    #[test]
    fn a_new_file_is_fetched_under_the_day_it_was_changed() {
        let dir = tempfile::tempdir().unwrap();
        let (found, notes) = ask(
            &server(Some(day(2026, 10, 15))),
            FetchMode::Online,
            dir.path(),
        );
        assert_eq!(
            found.unwrap(),
            Current::Fetch {
                path: dir.path().join("tn_09565-20261015.zip"),
                older: Vec::new(),
            }
        );
        assert!(notes.is_empty());
    }

    #[test]
    fn the_same_day_uses_the_cache() {
        let dir = tempfile::tempdir().unwrap();
        let kept = dir.path().join("tn_09565-20261015.zip");
        std::fs::write(&kept, b"zip").unwrap();
        let fetcher = server(Some(day(2026, 10, 15)));
        let (found, _) = ask(&fetcher, FetchMode::Online, dir.path());
        assert_eq!(found.unwrap(), Current::Cached(kept));
        // The one request was the check; nothing was downloaded.
        assert_eq!(fetcher.calls.get(), 1);
        assert!(fetcher.log.borrow().is_empty());
    }

    #[test]
    fn another_day_fetches_and_names_the_older_file() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("tn_09565-20260915.zip");
        std::fs::write(&old, b"old").unwrap();
        // A file of another Kreis is not an older version of this one.
        let other = dir.path().join("tn_09564-20260915.zip");
        std::fs::write(&other, b"other").unwrap();
        let (found, _) = ask(
            &server(Some(day(2026, 10, 15))),
            FetchMode::Online,
            dir.path(),
        );
        let Current::Fetch { path, older } = found.unwrap() else {
            panic!("not a fetch")
        };
        assert_eq!(path, dir.path().join("tn_09565-20261015.zip"));
        assert_eq!(older, vec![old.clone()]);
        drop_older(&older);
        assert!(!old.exists());
        assert!(other.exists());
    }

    #[test]
    fn offline_takes_the_newest_cached_and_asks_nothing() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["tn_09565-20260915.zip", "tn_09565-20261015.zip"] {
            std::fs::write(dir.path().join(name), b"zip").unwrap();
        }
        let fetcher = server(Some(day(2026, 11, 1)));
        let (found, _) = ask(&fetcher, FetchMode::Offline, dir.path());
        assert_eq!(
            found.unwrap(),
            Current::Cached(dir.path().join("tn_09565-20261015.zip"))
        );
        assert_eq!(fetcher.calls.get(), 0);
        assert_eq!(
            cached(dir.path(), "tn_09565", "zip"),
            Some(dir.path().join("tn_09565-20261015.zip"))
        );
        assert_eq!(cached(dir.path(), "tn_09564", "zip"), None);
    }

    #[test]
    fn offline_without_a_cached_file_is_the_offline_error() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = server(Some(day(2026, 10, 15)));
        let (found, _) = ask(&fetcher, FetchMode::Offline, dir.path());
        let error = found.unwrap_err().to_string();
        assert!(error.contains("offline and not cached"), "{error}");
        assert!(error.contains(URL), "{error}");
        assert_eq!(fetcher.calls.get(), 0);
    }

    #[test]
    fn a_404_is_believed_for_30_days() {
        let dir = tempfile::tempdir().unwrap();
        let gone = MapFetcher::default();
        let (found, _) = ask(&gone, FetchMode::Online, dir.path());
        assert_eq!(found.unwrap(), Current::Missing);
        assert!(dir.path().join("tn_09565.missing").exists());
        assert_eq!(gone.calls.get(), 1);
        crate::clock::with_fixed(in_days(29), || {
            let (found, _) = ask(&gone, FetchMode::Online, dir.path());
            assert_eq!(found.unwrap(), Current::Missing);
            assert_eq!(gone.calls.get(), 1, "not asked again");
        });
        crate::clock::with_fixed(in_days(31), || {
            // Offline the note holds whatever its age.
            let (found, _) = ask(&gone, FetchMode::Offline, dir.path());
            assert_eq!(found.unwrap(), Current::Missing);
            assert_eq!(gone.calls.get(), 1);
            // Online the server is asked again, and now it has the file.
            let back = server(Some(day(2026, 12, 1)));
            let (found, _) = ask(&back, FetchMode::Online, dir.path());
            assert!(matches!(found.unwrap(), Current::Fetch { .. }));
            assert_eq!(back.calls.get(), 1);
            assert!(!dir.path().join("tn_09565.missing").exists());
        });
    }

    #[test]
    fn a_failed_check_goes_on_with_the_cached_file() {
        let dir = tempfile::tempdir().unwrap();
        let down = server(Some(day(2026, 10, 15)));
        down.failing.borrow_mut().push(URL.to_string());
        // Without a cached file the failure stops the fetch.
        let (found, notes) = ask(&down, FetchMode::Online, dir.path());
        assert!(found.is_err());
        assert!(notes.is_empty());
        let kept = dir.path().join("tn_09565-20260915.zip");
        std::fs::write(&kept, b"zip").unwrap();
        let (found, notes) = ask(&down, FetchMode::Online, dir.path());
        assert_eq!(found.unwrap(), Current::Cached(kept));
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id(), "warning-file-check-failed");
        assert!(notes[0].is_warning());
        assert!(notes[0].contains("tn_09565.zip"), "{:?}", notes[0]);
    }

    #[test]
    fn a_server_without_last_modified_is_asked_again_after_30_days() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = server(None);
        let (found, _) = ask(&fetcher, FetchMode::Online, dir.path());
        let undated = dir.path().join("tn_09565-00000000.zip");
        assert_eq!(
            found.unwrap(),
            Current::Fetch {
                path: undated.clone(),
                older: Vec::new(),
            }
        );
        std::fs::write(&undated, b"zip").unwrap();
        crate::clock::with_fixed(in_days(29), || {
            let (found, _) = ask(&fetcher, FetchMode::Online, dir.path());
            assert_eq!(found.unwrap(), Current::Cached(undated.clone()));
        });
        crate::clock::with_fixed(in_days(31), || {
            let (found, _) = ask(&fetcher, FetchMode::Online, dir.path());
            assert_eq!(
                found.unwrap(),
                Current::Fetch {
                    path: undated.clone(),
                    older: Vec::new(),
                }
            );
        });
    }
}
