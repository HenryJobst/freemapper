//! The clock behind every date the project stores and every age the fetch compares.
//!
//! The project file records when each part was fetched, the credit line takes its year from
//! those dates, and two kept files of Rhineland-Palatinate are believed for 30 days. A test that
//! compares output byte for byte needs all of that to stand still, so the library asks this
//! module and never the system clock (the test `no_direct_clock_calls` holds it to that).
//!
//! By default the three functions read the system clock. [`with_fixed`] replaces it for the
//! calling thread; it exists only in tests and with the feature `testing`. The library starts no
//! thread of its own, so the calling thread is the whole fetch. Code that moves work to another
//! thread must take the clock with it.

use std::path::Path;
use std::time::{Duration, SystemTime};

use chrono::NaiveDate;

#[cfg(any(test, feature = "testing"))]
thread_local! {
    /// The instant [`with_fixed`] set for this thread, as UTC.
    static FIXED: std::cell::Cell<Option<chrono::NaiveDateTime>> =
        const { std::cell::Cell::new(None) };
}

/// The fixed instant of this thread, if a test set one.
#[cfg(any(test, feature = "testing"))]
fn fixed() -> Option<chrono::NaiveDateTime> {
    FIXED.get()
}

/// The day a record written now is dated with.
pub fn today() -> NaiveDate {
    #[cfg(any(test, feature = "testing"))]
    if let Some(at) = fixed() {
        return at.date();
    }
    chrono::Local::now().date_naive()
}

/// The instant a file's age is measured against.
pub fn now() -> SystemTime {
    #[cfg(any(test, feature = "testing"))]
    if let Some(at) = fixed() {
        return at.and_utc().into();
    }
    SystemTime::now()
}

/// How long ago the file at `path` was last written: [`now`] minus its modification time.
/// `None` when the file or its time cannot be read. A file dated after now has age zero, so a
/// file written under a fixed clock, which carries the real time, counts as fresh.
pub fn age_of(path: &Path) -> Option<Duration> {
    let written = std::fs::metadata(path).ok()?.modified().ok()?;
    Some(now().duration_since(written).unwrap_or(Duration::ZERO))
}

/// Runs `f` with the clock of this thread standing at `at` (read as UTC, so the result does not
/// depend on the time zone), and puts the earlier clock back afterwards, also when `f` panics.
#[cfg(any(test, feature = "testing"))]
pub fn with_fixed<R>(at: chrono::NaiveDateTime, f: impl FnOnce() -> R) -> R {
    struct Restore(Option<chrono::NaiveDateTime>);
    impl Drop for Restore {
        fn drop(&mut self) {
            FIXED.set(self.0);
        }
    }
    let _restore = Restore(FIXED.replace(Some(at)));
    f()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noon(year: i32, month: u32, day: u32) -> chrono::NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
    }

    #[test]
    fn the_fixed_clock_gives_its_day_and_instant() {
        let at = noon(2020, 1, 2);
        with_fixed(at, || {
            assert_eq!(today(), at.date());
            assert_eq!(now(), SystemTime::from(at.and_utc()));
        });
    }

    #[test]
    fn the_clock_runs_again_after_a_fixed_stretch() {
        let before = SystemTime::now();
        with_fixed(noon(2020, 1, 2), || {
            with_fixed(noon(2021, 3, 4), || {
                assert_eq!(today(), noon(2021, 3, 4).date())
            });
            assert_eq!(today(), noon(2020, 1, 2).date());
        });
        assert!(now() >= before);
    }

    #[test]
    fn a_panic_does_not_leave_the_clock_fixed() {
        let panicked = std::panic::catch_unwind(|| with_fixed(noon(2020, 1, 2), || panic!("stop")));
        assert!(panicked.is_err());
        assert!(fixed().is_none());
    }

    #[test]
    fn another_thread_keeps_the_system_clock() {
        with_fixed(noon(2020, 1, 2), || {
            let elsewhere = std::thread::spawn(today).join().unwrap();
            assert_ne!(elsewhere, noon(2020, 1, 2).date());
        });
    }

    /// The lines of `source` before its test module that ask the system clock directly.
    fn direct_clock_calls(source: &str) -> Vec<usize> {
        const CALLS: [&str; 4] = ["Local::now(", "Utc::now(", "SystemTime::now(", ".elapsed()"];
        source
            .lines()
            .take_while(|line| *line != "mod tests {")
            .enumerate()
            .filter(|(_, line)| !line.trim_start().starts_with("//"))
            .filter(|(_, line)| CALLS.iter().any(|call| line.contains(call)))
            .map(|(index, _)| index + 1)
            .collect()
    }

    #[test]
    fn the_scan_finds_calls_outside_the_test_module_only() {
        let source = "fn a() {\n    let t = std::time::SystemTime::now();\n}\n\
            // SystemTime::now() in a comment\nfn b() { started.elapsed() }\n\
            #[cfg(test)]\nmod tests {\n    fn c() { chrono::Local::now(); }\n}\n";
        assert_eq!(direct_clock_calls(source), [2, 5]);
    }

    /// Every date and age of the library crates comes from this module. The CLI crate keeps its
    /// own clock calls (run lock, update check); they write no project file.
    #[test]
    fn no_direct_clock_calls() {
        fn sources(dir: &Path, found: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    sources(&path, found);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    found.push(path);
                }
            }
        }
        // The library crates: this one, the engine, the readers and the adapters.
        const LIBRARY_CRATES: [&str; 4] = [
            "src",
            "../freemapper/src",
            "../freemapper-sources/src",
            "../adapters/de/src",
        ];
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files = Vec::new();
        for src in LIBRARY_CRATES {
            let before = files.len();
            sources(&manifest.join(src), &mut files);
            assert!(files.len() > before, "no sources under {src}");
        }
        assert!(files.len() > 60, "{} files", files.len());
        let mut calls = Vec::new();
        for file in files {
            if file == manifest.join("src").join("clock.rs") {
                continue;
            }
            let text = std::fs::read_to_string(&file).unwrap();
            for line in direct_clock_calls(&text) {
                calls.push(format!(
                    "{}:{line}",
                    file.strip_prefix(manifest).unwrap().display()
                ));
            }
        }
        assert!(
            calls.is_empty(),
            "ask freemapper_adapter_api::clock instead: {calls:?}"
        );
    }

    #[test]
    fn a_file_dated_after_the_clock_is_fresh() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kept");
        std::fs::write(&path, b"x").unwrap();
        // The file carries the real time, years after the fixed clock.
        with_fixed(noon(2020, 1, 2), || {
            assert_eq!(age_of(&path), Some(Duration::ZERO));
        });
    }

    #[test]
    fn age_is_the_clock_minus_the_modification_time() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kept");
        std::fs::write(&path, b"x").unwrap();
        let written = std::fs::metadata(&path).unwrap().modified().unwrap();
        let later =
            chrono::DateTime::<chrono::Utc>::from(written).naive_utc() + chrono::Duration::days(31);
        let age = with_fixed(later, || age_of(&path)).unwrap();
        assert_eq!(age, Duration::from_secs(31 * 86_400));
        assert_eq!(age_of(&dir.path().join("absent")), None);
    }
}
