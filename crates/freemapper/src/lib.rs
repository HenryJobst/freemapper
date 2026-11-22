// SPDX-License-Identifier: AGPL-3.0-or-later

//! freemapper builds OpenOrienteering Mapper starter projects from official geodata and OSM.
//!
//! Terms follow `docs/glossary.md`.

pub use freemapper_omap::{View, ViewError};
pub mod adapters;
pub mod area;
pub mod build;
pub mod clock;
pub mod crs;
pub mod derive;
pub mod fetch;
mod fsutil;
pub mod georef;
pub mod layers;
pub mod measure;
pub mod note;
pub mod ocd;
pub mod omap;
pub mod permits;
pub mod places;
pub mod probe;
pub mod project;
pub mod raster;
pub mod render;
pub mod sources;
pub mod space;
pub mod tiles;
