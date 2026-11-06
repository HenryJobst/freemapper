//! Georeferencing of a map: reference point, scale, declination, grivation and the transform
//! between projected coordinates and map coordinates.

use chrono::NaiveDate;

use chrono::Datelike;

use crate::crs::{
    EPSG_UTM33, LatLon, ProjectedPoint, grid_convergence_deg, point_scale_factor, utm33_forward,
};
use crate::omap::MapCoord;

#[derive(Debug, thiserror::Error)]
pub enum GeorefError {
    #[error("date {0} is outside the World Magnetic Model validity")]
    DateOutsideModel(NaiveDate),
}

/// Magnetic declination in degrees (east positive) at `p` on `date`, from the World Magnetic
/// Model at ellipsoid height 0.
pub fn magnetic_declination_deg(p: LatLon, date: NaiveDate) -> Result<f64, GeorefError> {
    use world_magnetic_model::GeomagneticField;
    use world_magnetic_model::uom::si::angle::degree;
    use world_magnetic_model::uom::si::f32::{Angle, Length};
    use world_magnetic_model::uom::si::length::meter;

    let outside = || GeorefError::DateOutsideModel(date);
    let ordinal = u16::try_from(date.ordinal()).map_err(|_| outside())?;
    let wmm_date = world_magnetic_model::time::Date::from_ordinal_date(date.year(), ordinal)
        .map_err(|_| outside())?;
    let field = GeomagneticField::new(
        Length::new::<meter>(0.0),
        Angle::new::<degree>(p.lat as f32),
        Angle::new::<degree>(p.lon as f32),
        wmm_date,
    )
    .map_err(|_| outside())?;
    Ok(f64::from(field.declination().get::<degree>()))
}

/// Rounds an angle to 0.01°, the precision OOM keeps for declination and grivation.
fn round_angle(deg: f64) -> f64 {
    (deg * 100.0).round() / 100.0
}

/// Rounds a scale factor to 1e-6.
fn round_scale_factor(factor: f64) -> f64 {
    (factor * 1e6).round() / 1e6
}

/// Georeferencing as written to an OOM file. The auxiliary (elevation) scale factor is 1, so
/// `grid_scale_factor` is also the combined scale factor.
#[derive(Debug, Clone, PartialEq)]
pub struct Georeferencing {
    pub scale_denominator: u32,
    pub epsg: u32,
    pub ref_geographic: LatLon,
    pub ref_projected: ProjectedPoint,
    pub declination_deg: f64,
    pub convergence_deg: f64,
    pub grid_scale_factor: f64,
}

impl Georeferencing {
    /// Georeferencing in EPSG:25833 with the map reference point at `ref_point`.
    pub fn new(
        ref_point: LatLon,
        scale_denominator: u32,
        declination_date: NaiveDate,
    ) -> Result<Self, GeorefError> {
        Ok(Self {
            scale_denominator,
            epsg: EPSG_UTM33,
            ref_geographic: ref_point,
            ref_projected: utm33_forward(ref_point),
            declination_deg: round_angle(magnetic_declination_deg(ref_point, declination_date)?),
            convergence_deg: grid_convergence_deg(ref_point),
            grid_scale_factor: round_scale_factor(point_scale_factor(ref_point)),
        })
    }

    /// Angle from grid north to magnetic north in degrees.
    pub fn grivation_deg(&self) -> f64 {
        round_angle(self.declination_deg - self.convergence_deg)
    }

    /// Metres in the projected CRS per millimetre on the map.
    fn metres_per_map_mm(&self) -> f64 {
        self.grid_scale_factor * f64::from(self.scale_denominator) / 1000.0
    }

    pub fn projected_to_map(&self, p: ProjectedPoint) -> MapCoord {
        let s = self.metres_per_map_mm();
        let (sin_g, cos_g) = self.grivation_deg().to_radians().sin_cos();
        let de = p.easting - self.ref_projected.easting;
        let dn = p.northing - self.ref_projected.northing;
        let x_mm = (de * cos_g - dn * sin_g) / s;
        let y_mm = -(de * sin_g + dn * cos_g) / s;
        MapCoord::new(
            (x_mm * 1000.0).round() as i32,
            (y_mm * 1000.0).round() as i32,
        )
    }

    pub fn map_to_projected(&self, c: MapCoord) -> ProjectedPoint {
        let s = self.metres_per_map_mm();
        let (sin_g, cos_g) = self.grivation_deg().to_radians().sin_cos();
        let x_mm = f64::from(c.x) / 1000.0;
        let y_mm = f64::from(c.y) / 1000.0;
        ProjectedPoint {
            easting: self.ref_projected.easting + s * (x_mm * cos_g - y_mm * sin_g),
            northing: self.ref_projected.northing + s * (-x_mm * sin_g - y_mm * cos_g),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 7).unwrap()
    }

    fn school() -> LatLon {
        LatLon {
            lat: 52.5401,
            lon: 13.8442,
        }
    }

    fn manual(
        declination_deg: f64,
        convergence_deg: f64,
        grid_scale_factor: f64,
    ) -> Georeferencing {
        Georeferencing {
            scale_denominator: 1000,
            epsg: EPSG_UTM33,
            ref_geographic: school(),
            ref_projected: ProjectedPoint {
                easting: 421_600.0,
                northing: 5_821_700.0,
            },
            declination_deg,
            convergence_deg,
            grid_scale_factor,
        }
    }

    fn offset(g: &Georeferencing, de: f64, dn: f64) -> ProjectedPoint {
        ProjectedPoint {
            easting: g.ref_projected.easting + de,
            northing: g.ref_projected.northing + dn,
        }
    }

    #[test]
    fn declination_strausberg_2026() {
        let d = magnetic_declination_deg(school(), date()).unwrap();
        assert!((4.0..7.0).contains(&d), "declination {d}");
    }

    #[test]
    fn new_computes_consistent_values() {
        let g = Georeferencing::new(school(), 1000, date()).unwrap();
        assert_eq!(g.epsg, EPSG_UTM33);
        assert!(
            (g.convergence_deg + 0.9175).abs() < 0.001,
            "{}",
            g.convergence_deg
        );
        assert!((g.grid_scale_factor - 0.999675).abs() < 1e-6);
        assert_eq!(
            g.declination_deg,
            (g.declination_deg * 100.0).round() / 100.0
        );
        let expected = ((g.declination_deg - g.convergence_deg) * 100.0).round() / 100.0;
        assert_eq!(g.grivation_deg(), expected);
    }

    #[test]
    fn ref_point_maps_to_origin() {
        let g = Georeferencing::new(school(), 1000, date()).unwrap();
        assert_eq!(g.projected_to_map(g.ref_projected), MapCoord::new(0, 0));
    }

    #[test]
    fn north_is_up_without_grivation() {
        let g = manual(0.0, 0.0, 1.0);
        assert_eq!(
            g.projected_to_map(offset(&g, 0.0, 100.0)),
            MapCoord::new(0, -100_000)
        );
        assert_eq!(
            g.projected_to_map(offset(&g, 50.0, 0.0)),
            MapCoord::new(50_000, 0)
        );
    }

    #[test]
    fn grivation_rotates_grid_north_left() {
        let g = manual(5.0, -1.0, 1.0);
        let c = g.projected_to_map(offset(&g, 0.0, 100.0));
        assert!(c.x < 0 && c.y < 0, "{c:?}");
        let expected = 100_000.0 * 6.0_f64.to_radians().sin();
        assert!((f64::from(c.x).abs() - expected).abs() <= 2.0, "{c:?}");
    }

    #[test]
    fn combined_factor_scales() {
        let g = manual(0.0, 0.0, 0.9996);
        let c = g.projected_to_map(offset(&g, 100.0, 0.0));
        assert_eq!(c.x, (100_000.0 / 0.9996_f64).round() as i32);
    }

    #[test]
    fn roundtrip_map_projected() {
        let g = Georeferencing::new(school(), 1000, date()).unwrap();
        for (de, dn) in [(12.3, -45.6), (-130.0, 140.0), (0.5, 0.25)] {
            let p = offset(&g, de, dn);
            let back = g.map_to_projected(g.projected_to_map(p));
            assert!(
                (back.easting - p.easting).abs() < 0.001,
                "{back:?} vs {p:?}"
            );
            assert!(
                (back.northing - p.northing).abs() < 0.001,
                "{back:?} vs {p:?}"
            );
        }
    }
}
