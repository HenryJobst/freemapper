// SPDX-License-Identifier: AGPL-3.0-or-later

//! The region adapter of Poland (ADR 0164): one national adapter, `polska`, whose OSM extracts
//! are those of the voivodeships. The crate knows the api crate and the source readers, and nothing of the engine.
//!
//! Terms follow `docs/glossary.md`.

pub mod polska;
pub mod rasters;
pub mod sheets;

pub use polska::{register, register_with_hosts};
