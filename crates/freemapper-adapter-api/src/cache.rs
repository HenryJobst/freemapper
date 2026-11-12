//! The cache shared by all projects, as an adapter sees it: a root folder with one folder per
//! provider. How much room the volume has left is the engine's business, not the adapter's.

use std::path::{Path, PathBuf};
use std::time::Duration;

/// Partial files older than this are left over from interrupted runs.
pub const STALE_PART_AGE: Duration = Duration::from_secs(3600);

/// The folder of the shared cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheDir {
    root: PathBuf,
}

impl CacheDir {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// The folder of the cache.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The folder of the provider that keeps its downloads under `name` (`lgb`, `lvermgeo_sh`).
    /// The name is the folder's: for four providers it is not their id, and caches in use hold
    /// several GB under these names. Only lower-case letters, digits and `_` make a name, so
    /// that nothing else can become part of a path.
    pub fn provider_dir(&self, name: &str) -> PathBuf {
        assert!(
            !name.is_empty()
                && name
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'),
            "{name:?} is no folder name of a provider"
        );
        self.root.join(name)
    }

    /// Folder of the cached OSM extracts.
    pub fn osm_dir(&self) -> PathBuf {
        self.root.join("osm")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_provider_folder_lies_directly_in_the_cache() {
        let cache = CacheDir::new(PathBuf::from("cache"));
        assert_eq!(
            cache.provider_dir("lvermgeo_sh"),
            Path::new("cache/lvermgeo_sh")
        );
        assert_eq!(cache.osm_dir(), Path::new("cache/osm"));
        assert_eq!(cache.root(), Path::new("cache"));
    }

    #[test]
    #[should_panic(expected = "no folder name of a provider")]
    fn a_path_is_no_provider_folder() {
        let _ = CacheDir::new(PathBuf::from("cache")).provider_dir("../x");
    }
}
