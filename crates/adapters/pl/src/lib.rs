//! The region adapter of Poland (ADR 0161): one national adapter, `polska`, whose OSM extracts
//! are those of the voivodeships. The crate knows the api crate and nothing of the engine.
//!
//! Terms follow `docs/glossary.md`.

pub mod polska;

pub use polska::{register, register_with_hosts};
