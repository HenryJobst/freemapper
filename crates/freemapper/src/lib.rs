//! freemapper builds OpenOrienteering Mapper starter projects from official geodata and OSM.
//!
//! Terms follow `docs/glossary.md`.

pub mod area;
pub mod build;
pub mod crs;
pub mod derive;
pub mod fetch;
mod fsutil;
pub mod georef;
pub mod omap;
pub mod places;
pub mod project;
pub mod raster;
pub mod sources;
pub mod tiles;
