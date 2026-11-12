//! What a raster fetch gets from the engine.

use chrono::NaiveDate;

use crate::cache::CacheDir;
use crate::fetch::{FetchMode, Fetcher};
use crate::note::Note;
use crate::registry::Registry;

/// The surroundings of one fetch: how to download, which regions there are, where the cache
/// lies, what day it is and where to report. An adapter asks nothing else of the engine; above all it never reads the
/// clock itself, so a test can fix the day.
pub struct AdapterEnv<'a> {
    pub fetcher: &'a dyn Fetcher,
    /// The regions the program knows, for the questions of `geometry` (which tiles a provider
    /// publishes, whether the area reaches into another region).
    pub regions: &'a Registry,
    pub cache: &'a CacheDir,
    pub mode: FetchMode,
    /// The day the records of this fetch are dated with.
    pub today: NaiveDate,
    /// The surface model does not fit on the disk: it is not downloaded, and what is stored or
    /// cached of it stays.
    pub skip_surface: bool,
    /// Progress lines and warnings.
    pub progress: &'a mut dyn FnMut(Note),
}
