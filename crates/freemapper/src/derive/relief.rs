//! Local relief template from the DTM: the terrain minus its smoothed mean (an idea of
//! omapmaker's local relief layer). Banks, knolls, hollows and ditches stand out; the general
//! fall of the ground does not.

use super::contours::gaussian_smooth;
use crate::raster::Raster;

/// Heights at or beyond this above or below the mean are white or black.
pub const RELIEF_SPAN_M: f32 = 1.0;

/// The DTM minus its Gaussian mean with `sigma_m` metres; `None` where the DTM has no data.
pub fn local_relief(dtm: &Raster<f32>, sigma_m: f64) -> Raster<Option<f32>> {
    let mean = gaussian_smooth(dtm, sigma_m);
    Raster {
        grid: dtm.grid,
        data: dtm
            .data
            .iter()
            .zip(&mean.data)
            .map(|(z, m)| (!z.is_nan() && !m.is_nan()).then(|| z - m))
            .collect(),
    }
}

/// The relief as grey: the mean 128, `span_m` and more above white, below black.
pub fn relief_shade(relief: &Raster<Option<f32>>, span_m: f32) -> Raster<Option<u8>> {
    Raster {
        grid: relief.grid,
        data: relief
            .data
            .iter()
            .map(|r| r.map(|r| (128.0 + 127.5 * (r / span_m).clamp(-1.0, 1.0)).floor() as u8))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raster::GridSpec;

    fn dtm(size: u32, z: impl Fn(f64, f64) -> f32) -> Raster<f32> {
        let grid = GridSpec {
            west: 0.0,
            north: f64::from(size),
            pixel_size: 1.0,
            width: size,
            height: size,
        };
        let mut r = Raster::filled(grid, 0.0);
        for y in 0..size {
            for x in 0..size {
                r.set(x, y, z(f64::from(x), f64::from(y)));
            }
        }
        r
    }

    #[test]
    fn local_relief_of_a_plane_is_zero() {
        let relief = local_relief(
            &dtm(41, |x, y| 30.0 + 0.1 * x as f32 + 0.05 * y as f32),
            5.0,
        );
        // Away from the border the mean of a plane is the plane.
        assert!(relief.get(20, 20).unwrap().abs() < 1e-3);
    }

    #[test]
    fn a_bump_is_positive_and_a_pit_negative() {
        let bump = |cx: f64, sign: f32| {
            move |x: f64, y: f64| {
                let d2 = (x - cx).powi(2) + (y - 20.0).powi(2);
                30.0 + sign * 2.0 * (-d2 / 8.0).exp() as f32
            }
        };
        assert!(
            local_relief(&dtm(41, bump(20.0, 1.0)), 5.0)
                .get(20, 20)
                .unwrap()
                > 1.0
        );
        assert!(
            local_relief(&dtm(41, bump(20.0, -1.0)), 5.0)
                .get(20, 20)
                .unwrap()
                < -1.0
        );
    }

    #[test]
    fn relief_shade_maps_span_to_full_range() {
        let grid = GridSpec {
            west: 0.0,
            north: 1.0,
            pixel_size: 1.0,
            width: 5,
            height: 1,
        };
        let relief = Raster {
            grid,
            data: vec![Some(-2.0), Some(-1.0), Some(0.0), Some(1.0), None],
        };
        assert_eq!(
            relief_shade(&relief, 1.0).data,
            [Some(0), Some(0), Some(128), Some(255), None]
        );
    }

    #[test]
    fn nan_stays_none() {
        let mut r = dtm(11, |_, _| 30.0);
        r.set(5, 5, f32::NAN);
        let relief = local_relief(&r, 2.0);
        assert_eq!(relief.get(5, 5), None);
        assert!(relief.get(4, 5).is_some());
    }
}
