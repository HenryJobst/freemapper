//! Lengths on paper and what they are on the ground and in map coordinates.
//!
//! A rule of the build names a length in one of three units: millimetres on the printed sheet
//! ([`PaperMm`]), millimetres of a symbol as the norm draws it ([`SymbolMm`]) or metres on the
//! ground. The two paper units differ by the symbol enlargement of the map kind
//! ([`MapKind::symbol_enlargement`]): a symbol measure grows with the symbols, a paper measure
//! does not. The type of a constant says which it is, and a symbol measure cannot be converted
//! without the enlargement ([`SymbolScale`]).

use std::ops::{Add, Div};

use crate::omap::SymbolSet;
use crate::project::MapKind;

/// Map coordinates (1/1000 mm on paper) per millimetre on paper.
const MAP_UNITS_PER_MM: f64 = 1000.0;

/// Map coordinates (1/1000 mm on paper) per metre on the ground at `scale_denominator`.
pub fn map_units_per_metre(scale_denominator: u32) -> f64 {
    1e6 / f64::from(scale_denominator)
}

/// Metres on the ground per millimetre on paper at `scale_denominator`.
pub fn metres_per_mm(scale_denominator: u32) -> f64 {
    f64::from(scale_denominator) / 1000.0
}

/// The scale of a map and the enlargement of its symbols: what a [`SymbolMm`] needs to become a
/// length on the sheet or on the ground.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SymbolScale {
    scale_denominator: u32,
    enlargement: f64,
}

impl SymbolScale {
    /// The scale of a map of `kind` at `scale_denominator`.
    pub fn new(kind: MapKind, scale_denominator: u32) -> Self {
        Self {
            scale_denominator,
            enlargement: kind.symbol_enlargement(scale_denominator),
        }
    }

    /// The scale of a map at `scale_denominator` drawn with `symbol_set`, for a rule that gets
    /// the symbol set and not the map kind. [`MapKind::symbol_set`] builds the set with the
    /// enlargement of the kind, so both ways give the same scale.
    pub fn drawn_with(symbol_set: &SymbolSet, scale_denominator: u32) -> Self {
        Self {
            scale_denominator,
            enlargement: symbol_set.enlargement(),
        }
    }

    pub fn scale_denominator(self) -> u32 {
        self.scale_denominator
    }

    /// The factor the symbols are enlarged by.
    pub fn enlargement(self) -> f64 {
        self.enlargement
    }

    /// Metres on the ground per millimetre of a symbol at 100 %: the enlargement times
    /// [`metres_per_mm`].
    pub fn metres_per_symbol_mm(self) -> f64 {
        self.enlargement * metres_per_mm(self.scale_denominator)
    }

    /// Map coordinates per millimetre of a symbol at 100 %.
    pub fn map_units_per_symbol_mm(self) -> f64 {
        MAP_UNITS_PER_MM * self.enlargement
    }
}

/// A length on the printed sheet (mm), the same whatever the size of the symbols.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PaperMm(pub f64);

impl PaperMm {
    /// The length on the ground (metres) at `scale_denominator`.
    pub fn metres(self, scale_denominator: u32) -> f64 {
        self.0 * metres_per_mm(scale_denominator)
    }

    /// The length in map coordinates.
    pub fn map_units(self) -> f64 {
        self.0 * MAP_UNITS_PER_MM
    }

    /// What `metres` on the ground are on the sheet at `scale_denominator`.
    pub fn of_metres(metres: f64, scale_denominator: u32) -> Self {
        Self(metres * 1000.0 / f64::from(scale_denominator))
    }
}

/// A length of a symbol as the norm draws it at 100 % (mm): on the sheet it is enlarged like
/// the symbols.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SymbolMm(pub f64);

impl SymbolMm {
    /// The length on the sheet of a map at `scale`.
    pub fn on_paper(self, scale: SymbolScale) -> PaperMm {
        PaperMm(self.0 * scale.enlargement)
    }

    /// The length on the ground (metres) of a map at `scale`.
    pub fn metres(self, scale: SymbolScale) -> f64 {
        self.on_paper(scale).metres(scale.scale_denominator)
    }

    /// The length in map coordinates of a map at `scale`.
    pub fn map_units(self, scale: SymbolScale) -> f64 {
        self.on_paper(scale).map_units()
    }
}

/// An area on the printed sheet (mm²), the same whatever the size of the symbols.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PaperMm2(pub f64);

impl PaperMm2 {
    /// The area on the ground (m²) at `scale_denominator`.
    pub fn square_metres(self, scale_denominator: u32) -> f64 {
        let metres_per_mm = metres_per_mm(scale_denominator);
        self.0 * metres_per_mm * metres_per_mm
    }
}

/// An area of a symbol as the norm draws it at 100 % (mm²).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SymbolMm2(pub f64);

impl SymbolMm2 {
    /// The area on the ground (m²) of a map at `scale`.
    pub fn square_metres(self, scale: SymbolScale) -> f64 {
        self.0 * scale.metres_per_symbol_mm().powi(2)
    }
}

impl Add for SymbolMm {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Div<f64> for SymbolMm {
    type Output = Self;

    fn div(self, divisor: f64) -> Self {
        Self(self.0 / divisor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_paper_measure_is_the_same_at_every_symbol_size() {
        // 0.5 mm at 1:4000 are 2 m, whatever the map kind.
        assert_eq!(PaperMm(0.5).metres(4000), 2.0);
        assert_eq!(PaperMm(0.5).metres(1000), 0.5);
        assert_eq!(PaperMm(0.5).map_units(), 500.0);
        assert_eq!(PaperMm::of_metres(200.0, 4000), PaperMm(50.0));
    }

    #[test]
    fn a_symbol_measure_grows_with_the_symbols() {
        let sprint = SymbolScale::new(MapKind::Sprint, 4000);
        let school = SymbolScale::new(MapKind::Schoolyard, 1000);
        let forest = SymbolScale::new(MapKind::Forest, 7500);
        assert_eq!(SymbolMm(0.4).on_paper(sprint), PaperMm(0.4));
        assert_eq!(SymbolMm(0.4).on_paper(school), PaperMm(0.5));
        assert_eq!(SymbolMm(0.4).on_paper(forest), PaperMm(0.8));
        // 0.4 mm of a symbol: 1.6 m on a sprint map, 0.5 m on a school map, 6 m on a forest map
        // at 1:7500.
        assert_eq!(SymbolMm(0.4).metres(sprint), 1.6);
        assert_eq!(SymbolMm(0.4).metres(school), 0.5);
        assert_eq!(SymbolMm(0.4).metres(forest), 6.0);
        assert_eq!(SymbolMm(0.4).map_units(school), 500.0);
    }

    #[test]
    fn the_symbol_set_of_a_kind_carries_the_enlargement_of_the_kind() {
        for kind in [MapKind::Schoolyard, MapKind::Sprint, MapKind::Forest] {
            for &scale in kind.allowed_scales() {
                assert_eq!(
                    SymbolScale::drawn_with(&kind.symbol_set(scale), scale),
                    SymbolScale::new(kind, scale),
                    "{kind:?} 1:{scale}"
                );
            }
        }
    }

    #[test]
    fn an_area_grows_with_the_square() {
        // 1.5 mm² at 1:4000 are 24 m²; a symbol's 1 mm² on a school map is (1.25 m)².
        assert_eq!(PaperMm2(1.5).square_metres(4000), 24.0);
        let school = SymbolScale::new(MapKind::Schoolyard, 1000);
        assert_eq!(SymbolMm2(1.0).square_metres(school), 1.5625);
    }

    #[test]
    fn a_metre_on_the_ground_in_map_coordinates() {
        assert_eq!(map_units_per_metre(4000), 250.0);
        assert_eq!(map_units_per_metre(1000), 1000.0);
    }

    #[test]
    fn symbol_measures_add_up_before_they_are_enlarged() {
        let school = SymbolScale::new(MapKind::Schoolyard, 1000);
        assert_eq!(
            (SymbolMm(0.15) + SymbolMm(0.4) / 2.0).on_paper(school),
            PaperMm((0.15 + 0.2) * 1.25)
        );
        assert_eq!(school.metres_per_symbol_mm(), 1.25);
        assert_eq!(school.map_units_per_symbol_mm(), 1250.0);
    }
}
