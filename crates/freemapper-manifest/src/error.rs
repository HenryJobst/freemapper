//! What is wrong with a manifest, with the place and a stable code.

use std::fmt;
use std::path::PathBuf;

/// One finding of the parser or the validator.
///
/// The codes are stable; a text catalog words them later. `manifest.*` codes are about the file
/// itself, `policy.*` codes about where its URLs lead:
///
/// | Code | Meaning |
/// |---|---|
/// | `manifest.too-large` | the file is larger than [`crate::MAX_MANIFEST_BYTES`] |
/// | `manifest.syntax` | not TOML (a duplicated table is one case) |
/// | `manifest.format` | `format` is missing or not a format this program reads |
/// | `manifest.unknown-key` | a key the schema does not have, or not for this `kind` |
/// | `manifest.missing-key` | a required key is missing |
/// | `manifest.type` | a value of the wrong type |
/// | `manifest.id` | an id that does not match `[a-z][a-z0-9]{1,31}`, or not the folder's name |
/// | `manifest.min-freemapper` | not a version `x.y.z` |
/// | `manifest.crs` | a CRS the key does not allow |
/// | `manifest.names` | an empty or overlong name |
/// | `manifest.value` | a value outside its range or pattern |
/// | `manifest.file-name` | a file name that is no plain name in the folder |
/// | `manifest.geofabrik` | not a path of an extract on the Geofabrik server |
/// | `manifest.osm` | the extracts do not fit together |
/// | `manifest.licence` | a licence outside the list, or `other` without its text |
/// | `manifest.attribution` | a credit that is empty, too long or has control characters |
/// | `manifest.kind` | an unknown `kind`, or `kind` and `backend` that contradict |
/// | `manifest.date-kind` | an unknown `kind` of a date lookup |
/// | `manifest.normalise` | an unknown `normalise` preset |
/// | `manifest.params` | a request parameter the manifest may not set |
/// | `manifest.too-many-urls` | more than [`crate::MAX_URLS`] URLs |
/// | `manifest.empty` | neither `[alkis]` nor `[orthophoto]` |
/// | `policy.url` | a URL that is not `https` on port 443 with a DNS name and without a query |
/// | `policy.host` | an entry of `[net] hosts` that is no DNS name |
/// | `policy.host-not-declared` | a URL whose host is not in `[net] hosts` |
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestError {
    pub code: &'static str,
    /// The manifest file.
    pub path: PathBuf,
    /// The line in the file, from 1; 0 when the finding has no place.
    pub line: usize,
    /// The key the finding is about, with its tables (`adapter.id`); empty for the whole file.
    pub key: String,
    pub message: String,
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())?;
        if self.line > 0 {
            write!(f, ":{}", self.line)?;
        }
        if !self.key.is_empty() {
            write!(f, ": {}", self.key)?;
        }
        write!(f, ": {} [{}]", self.message, self.code)
    }
}

impl std::error::Error for ManifestError {}
