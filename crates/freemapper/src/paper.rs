//! Paper formats (ISO 216) and the ground size a sheet covers at a map scale.

use freemapper_contract::tags::{ORIENTATION_LANDSCAPE, ORIENTATION_PORTRAIT};
use serde::{Deserialize, Serialize};

/// A paper format of the A series.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperFormat {
    A6,
    A5,
    A4,
    A3,
    A2,
}

impl PaperFormat {
    /// Every format, smallest first.
    pub const ALL: [PaperFormat; 5] = [Self::A6, Self::A5, Self::A4, Self::A3, Self::A2];

    pub fn name(self) -> &'static str {
        match self {
            Self::A6 => "A6",
            Self::A5 => "A5",
            Self::A4 => "A4",
            Self::A3 => "A3",
            Self::A2 => "A2",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|f| f.name() == name)
    }
}

/// How the sheet lies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    Landscape,
    Portrait,
}

impl Orientation {
    pub fn name(self) -> &'static str {
        match self {
            Self::Landscape => ORIENTATION_LANDSCAPE,
            Self::Portrait => ORIENTATION_PORTRAIT,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            ORIENTATION_LANDSCAPE => Some(Self::Landscape),
            ORIENTATION_PORTRAIT => Some(Self::Portrait),
            _ => None,
        }
    }
}

/// Sheet size in millimetres as (width, height).
pub fn size_mm(format: PaperFormat, orientation: Orientation) -> (f64, f64) {
    let (w, h) = match format {
        PaperFormat::A6 => (105.0, 148.0),
        PaperFormat::A5 => (148.0, 210.0),
        PaperFormat::A4 => (210.0, 297.0),
        PaperFormat::A3 => (297.0, 420.0),
        PaperFormat::A2 => (420.0, 594.0),
    };
    match orientation {
        Orientation::Portrait => (w, h),
        Orientation::Landscape => (h, w),
    }
}

/// Ground size in metres as (width, height) of a sheet at 1:`scale_denominator`.
pub fn ground_size_m(
    format: PaperFormat,
    orientation: Orientation,
    scale_denominator: u32,
) -> (f64, f64) {
    let (w, h) = size_mm(format, orientation);
    let k = f64::from(scale_denominator) / 1000.0;
    (w * k, h * k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a4_portrait_at_15000_is_3150_by_4455_m() {
        let size = ground_size_m(PaperFormat::A4, Orientation::Portrait, 15000);
        assert_eq!(size, (3150.0, 4455.0));
    }

    #[test]
    fn a6_landscape_at_4000_is_592_by_420_m() {
        let size = ground_size_m(PaperFormat::A6, Orientation::Landscape, 4000);
        assert_eq!(size, (592.0, 420.0));
    }

    #[test]
    fn landscape_swaps_the_sides() {
        for f in PaperFormat::ALL {
            let (pw, ph) = size_mm(f, Orientation::Portrait);
            assert_eq!(size_mm(f, Orientation::Landscape), (ph, pw));
        }
    }

    #[test]
    fn names_round_trip() {
        for f in PaperFormat::ALL {
            assert_eq!(PaperFormat::from_name(f.name()), Some(f));
        }
        for o in [Orientation::Landscape, Orientation::Portrait] {
            assert_eq!(Orientation::from_name(o.name()), Some(o));
        }
    }

    #[test]
    fn the_names_are_the_reserved_words_of_the_contract() {
        let papers: Vec<_> = PaperFormat::ALL.iter().map(|f| f.name()).collect();
        assert_eq!(papers, freemapper_contract::tags::PAPERS);
        let orientations = [Orientation::Landscape, Orientation::Portrait].map(Orientation::name);
        assert_eq!(orientations, freemapper_contract::tags::ORIENTATIONS);
    }

    #[test]
    fn unknown_name_is_none() {
        assert_eq!(PaperFormat::from_name("A7"), None);
        assert_eq!(Orientation::from_name("square"), None);
    }
}
