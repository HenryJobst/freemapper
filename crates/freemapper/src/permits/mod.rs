//! The permit notes of a map area: who protects it, who runs it, and whom a person asks
//! (ADR 0116, `docs/superpowers/specs/2026-11-13-permit-notes-design.md`).

pub mod fetch;
pub mod notes;
pub mod osm;
pub mod store;

use std::path::PathBuf;

use crate::project::ProjectError;

/// A source of the permit notes that gave nothing: its id (`vg250`, `bfn`, `osm`) and, when it
/// failed, why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Missing {
    pub source: String,
    /// The failure; `None` when the source was not read at all (never fetched, no extract).
    pub reason: Option<String>,
}

impl std::fmt::Display for Missing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.reason {
            Some(reason) => write!(f, "{}: {reason}", self.source),
            None => write!(f, "{}: not read", self.source),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PermitsError {
    #[error(transparent)]
    Project(#[from] ProjectError),
    #[error("{}", crate::space::io_error_text(.0, .1))]
    Io(PathBuf, std::io::Error),
    /// A stored file of the permit notes cannot be read back.
    #[error("{}: {message}; fetch the permit notes again online", path.display())]
    Stored { path: PathBuf, message: String },
    /// None of the three sources gave anything.
    #[error(
        "no source of the permit notes could be read ({}); fetch them online first",
        missing.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ")
    )]
    NothingRead { missing: Vec<Missing> },
}
