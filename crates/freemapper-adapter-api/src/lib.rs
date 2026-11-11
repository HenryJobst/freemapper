//! What a region adapter of freemapper works with (ADR 0082).
//!
//! The engine, the generic readers (`freemapper-sources`) and the adapters share these types. The
//! crate knows no state and no service, and it has no HTTP client: `freemapper` brings the
//! fetcher that talks to the network.
//!
//! Terms follow `docs/glossary.md`.

pub mod adapter;
pub mod cache;
pub mod credit;
pub mod env;
pub mod error;
pub mod estimate;
pub mod fetch;
pub mod fs;
pub mod info;
pub mod note;
pub mod project;
pub mod records;
pub mod registry;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
