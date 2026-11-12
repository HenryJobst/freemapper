//! The adapter manifest of freemapper (ADR 0082): the TOML file that declares the standard
//! services of a region adapter, format 1.
//!
//! A manifest is untrusted input: people copy them from forums and repositories. [`parse`]
//! reads the text into a strict schema (an unknown key is an error), checks every value against
//! a closed list, a range or a pattern, and checks every URL against the rules of
//! `freemapper_adapter_api::policy` and the hosts the manifest declares. It reads no other file
//! and makes no request.
//!
//! Terms follow `docs/glossary.md`.

use std::path::Path;

pub mod adapter;
pub mod alkis;
pub mod boundary;
pub mod dates;
pub mod error;
pub mod estimate;
pub mod extras;
pub mod guard;
pub mod loader;
pub mod model;
pub mod rasters;
mod schema;
mod validate;

pub use error::ManifestError;
pub use model::Manifest;
pub use validate::MAX_ANSWER_BYTES;

/// The name of the manifest file in the folder of an adapter.
pub const MANIFEST_FILE: &str = "manifest.toml";
/// The format this program writes and reads.
pub const FORMAT: i64 = 1;
/// The largest manifest read, in bytes.
pub const MAX_MANIFEST_BYTES: usize = 64 * 1024;
/// The most URLs one manifest may hold.
pub const MAX_URLS: usize = 64;

/// Reads and checks the manifest `text` of the adapter folder `folder`, whose name is the id of
/// the adapter. All findings of the validator come back at once; a file that is not TOML or
/// not of the schema gives the one finding of the parser.
pub fn parse(text: &str, folder: &Path) -> Result<Manifest, Vec<ManifestError>> {
    validate::parse(text, folder)
}
