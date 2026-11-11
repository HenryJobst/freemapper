//! The app side of the self-update (spec 3.8): the HTTP source, the app's identity and keys, and
//! whether this build may update itself. The pure parts live in `freemapper-update`.

mod source;

use std::sync::LazyLock;

use freemapper_update::{App, Version};

pub use source::HttpUpdateSource;

/// The accepted minisign public keys, one per line; the `untrusted comment:` lines are skipped.
const KEY_FILE: &str = include_str!("../../../../assets/update/minisign.pub");

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
pub fn release_build() -> bool {
    option_env!("FREEMAPPER_DIST") == Some("github-release")
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
}
