// SPDX-License-Identifier: AGPL-3.0-or-later

//! The region adapter of Czechia (ADR 0197): one national adapter, `cesko`, whose OSM extract is
//! the one of Geofabrik for the whole country. The crate knows the api crate and the source
//! readers, and nothing of the engine.
//!
//! Terms follow `docs/glossary.md`.

pub mod cadastre;
pub mod cesko;
pub mod rasters;

pub use cesko::{register, register_with_hosts};
