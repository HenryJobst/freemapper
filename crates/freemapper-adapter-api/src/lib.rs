//! What a region adapter of freemapper works with (ADR 0082).
//!
//! The engine, the generic readers (`freemapper-sources`) and the adapters share these types. The
//! crate knows no state and no service, and it has no HTTP client: `freemapper` brings the
//! fetcher that talks to the network.
//!
//! Terms follow `docs/glossary.md`.

pub mod error;
pub mod fetch;
pub mod fs;
pub mod note;
pub mod records;
