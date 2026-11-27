// SPDX-License-Identifier: AGPL-3.0-or-later

//! The region adapter of Sweden (ADR 0202): one national adapter, `sweden`, whose addresses are
//! its manifest (`manifests/sweden/manifest.toml`, ADR 0189) and whose code reads what no
//! manifest table says: the STAC catalogues of Lantmäteriet, the terrain model by range, the
//! point clouds and the land cover. The crate knows the api crate and the source readers, and
//! nothing of the engine.
//!
//! Terms follow `docs/glossary.md`.

mod account;
pub mod copc;
pub mod marktacke;
pub mod stac_terrain;
pub mod sweden;

pub use sweden::{SWEDEN, register, register_with_hosts};
