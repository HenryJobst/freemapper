//! The app side of the self-update (spec 3.8): the HTTP source, the app's identity and keys,
//! whether this build may update itself, the settings folder, the `update` command and the daily
//! check after network commands. The pure parts live in `freemapper-update`.

pub mod background;
pub mod command;
mod source;

use std::path::PathBuf;
use std::sync::LazyLock;

use freemapper_update::{App, Version};

pub use source::HttpUpdateSource;

/// The accepted minisign public keys, one per line; the `untrusted comment:` lines are skipped.
const KEY_FILE: &str = include_str!("../../../../assets/update/minisign.pub");

/// Whether `release.yml` built this program: it sets `FREEMAPPER_DIST=github-release` (spec 3.6).
const RELEASE: bool = is_release(option_env!("FREEMAPPER_DIST"));

// A release build with the test key would accept updates signed by a key whose secret half is
// public. The key file says "TEST KEY" until task B6 puts the release key there.
const _: () = assert!(
    key_file_allowed(RELEASE, KEY_FILE),
    "FREEMAPPER_DIST=github-release, but assets/update/minisign.pub still holds the TEST KEY"
);

/// `dist` is `github-release`. A `const fn`, so the build can check it.
const fn is_release(dist: Option<&str>) -> bool {
    match dist {
        Some(dist) => bytes_equal(dist.as_bytes(), b"github-release"),
        None => false,
    }
}

/// Whether a build of this kind may carry this key file: a release build never one with the test key.
const fn key_file_allowed(release: bool, key_file: &str) -> bool {
    !release || !contains(key_file.as_bytes(), b"TEST KEY")
}

const fn bytes_equal(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// Whether `needle` occurs in `haystack`.
const fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    let mut start = 0;
    while start + needle.len() <= haystack.len() {
        let mut i = 0;
        while i < needle.len() && haystack[start + i] == needle[i] {
            i += 1;
        }
        if i == needle.len() {
            return true;
        }
        start += 1;
    }
    false
}

static KEYS: LazyLock<Vec<&'static str>> = LazyLock::new(|| key_lines(KEY_FILE));

/// The key lines of a key file: every non-empty line that is not a comment.
fn key_lines(text: &'static str) -> Vec<&'static str> {
    text.lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty() && !line.starts_with("untrusted comment:") && !line.starts_with('#')
        })
        .collect()
}

/// freemapper as the update code sees it: name, repository, keys and the running version.
pub fn freemapper_app() -> App {
    App {
        name: "freemapper",
        repo_url: "https://github.com/HenryJobst/freemapper",
        keys: KEYS.as_slice(),
        current: env!("CARGO_PKG_VERSION")
            .parse::<Version>()
            .expect("the package version is x.y.z"),
    }
}

/// Whether this program is a release build of this repository (spec 3.6): `release.yml` sets
/// `FREEMAPPER_DIST=github-release` while building. A build from source neither checks nor updates.
/// The CLI tests count as a release build when they name a stub server ([`test_base`]).
pub fn release_build() -> bool {
    RELEASE || test_base().is_some()
}

/// Tests only: the stub server that `FREEMAPPER_UPDATE_TEST_BASE` names. Read only by a debug
/// build with the `test-stub` feature; any other build has no such switch.
#[cfg(all(debug_assertions, feature = "test-stub"))]
fn test_base() -> Option<String> {
    std::env::var("FREEMAPPER_UPDATE_TEST_BASE")
        .ok()
        .filter(|base| !base.is_empty())
}

#[cfg(not(all(debug_assertions, feature = "test-stub")))]
fn test_base() -> Option<String> {
    None
}

/// The source of updates: GitHub, or in the CLI tests the stub server.
pub fn update_source() -> HttpUpdateSource {
    #[cfg(all(debug_assertions, feature = "test-stub"))]
    if let Some(base) = test_base() {
        return HttpUpdateSource::stub(&base);
    }
    HttpUpdateSource::new()
}

/// The folder of `settings.toml` (spec 3.4): `FREEMAPPER_CONFIG_DIR` if set (the tests use it),
/// else the system's configuration folder + `freemapper`.
pub fn settings_dir() -> Option<PathBuf> {
    settings_dir_from(
        std::env::var_os("FREEMAPPER_CONFIG_DIR"),
        dirs::config_dir(),
    )
}

fn settings_dir_from(
    overridden: Option<std::ffi::OsString>,
    system: Option<PathBuf>,
) -> Option<PathBuf> {
    match overridden {
        Some(dir) if !dir.is_empty() => Some(PathBuf::from(dir)),
        _ => system.map(|dir| dir.join("freemapper")),
    }
}

/// `FREEMAPPER_UPDATE`, which overrides the stored mode for one process (spec 3.4). Read once by
/// the caller and handed to `freemapper_update::load_mode`.
pub fn mode_from_env() -> Option<String> {
    std::env::var("FREEMAPPER_UPDATE").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_keys_are_the_key_lines_of_the_key_file() {
        let app = freemapper_app();
        assert_eq!(app.name, "freemapper");
        assert_eq!(app.current.to_string(), env!("CARGO_PKG_VERSION"));
        assert_eq!(
            app.keys,
            ["RWQr6TpLSQ74CbpwfXt6elefnjKJxHRj6tEawt4fudtGpb7yLegwOqX+"]
        );
        assert_eq!(
            key_lines("untrusted comment: a\n\nKEY1\n# note\n KEY2 \n"),
            ["KEY1", "KEY2"]
        );
    }

    #[test]
    fn a_release_build_refuses_the_test_key() {
        assert!(is_release(Some("github-release")));
        assert!(!is_release(Some("github")) && !is_release(Some("")) && !is_release(None));
        let test_key =
            "untrusted comment: TEST KEY, replaced by the release key in task B6\nRWQ...\n";
        assert!(!key_file_allowed(true, test_key));
        assert!(key_file_allowed(false, test_key));
        assert!(key_file_allowed(
            true,
            "untrusted comment: minisign public key 1234\nRWQ...\n"
        ));
        assert!(contains(b"ab TEST KEY", b"TEST KEY") && !contains(b"TEST KE", b"TEST KEY"));
    }

    #[test]
    fn settings_dir_is_the_override_or_the_system_folder() {
        let system = Some(PathBuf::from("/conf"));
        assert_eq!(
            settings_dir_from(Some("/elsewhere".into()), system.clone()),
            Some(PathBuf::from("/elsewhere"))
        );
        assert_eq!(
            settings_dir_from(Some("".into()), system.clone()),
            Some(PathBuf::from("/conf/freemapper"))
        );
        assert_eq!(
            settings_dir_from(None, system),
            Some(PathBuf::from("/conf/freemapper"))
        );
        assert_eq!(settings_dir_from(None, None), None);
    }
}
