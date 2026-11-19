// SPDX-License-Identifier: AGPL-3.0-or-later

//! Building map objects from source data, following the symbol mapping rules.

pub mod assembly;
pub mod barrier_gaps;
pub mod barriers;
#[cfg(test)]
mod berlin;
pub mod buildings;
pub mod candidate_layers;
pub mod candidates;
pub mod canopy;
pub mod cliffs;
pub mod conflicts;
pub mod content;
pub mod contours;
pub mod control_sites;
pub mod dedup;
pub mod firm_paths;
pub mod geometry;
pub mod landforms;
pub mod light;
pub mod line_cuts;
pub mod line_join;
pub mod notches;
pub mod open_land;
pub mod out_of_bounds;
pub mod passages;
pub mod paved;
pub mod paved_borders;
pub mod peripheral;
pub mod points;
pub mod sets;
pub mod source_block;
pub mod stairs;
pub mod state;
pub mod symbol_measures;
pub mod tagged;
pub mod ticks;
pub mod training;
pub mod vegetation;
pub mod walls;
pub mod water;
