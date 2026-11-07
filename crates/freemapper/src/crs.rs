//! Coordinate reference systems: ETRS89 / UTM zone 33N (EPSG:25833).

/// Geographic position in degrees (ETRS89, treated as identical to WGS84).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LatLon {
    pub lat: f64,
    pub lon: f64,
}

/// Projected position (easting / northing) in metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectedPoint {
    pub easting: f64,
    pub northing: f64,
}

/// Axis-parallel rectangle in projected coordinates (metres).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min_e: f64,
    pub min_n: f64,
    pub max_e: f64,
    pub max_n: f64,
}

impl BoundingBox {
    /// The box grown by `m` metres on every side.
    pub fn buffered(&self, m: f64) -> Self {
        Self {
            min_e: self.min_e - m,
            min_n: self.min_n - m,
            max_e: self.max_e + m,
            max_n: self.max_n + m,
        }
    }

    /// The smallest box with corners on multiples of `step` that contains this box.
    pub fn snapped_outward(&self, step: f64) -> Self {
        Self {
            min_e: (self.min_e / step).floor() * step,
            min_n: (self.min_n / step).floor() * step,
            max_e: (self.max_e / step).ceil() * step,
            max_n: (self.max_n / step).ceil() * step,
        }
    }

    /// The common part of both boxes, if it is not empty.
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let b = Self {
            min_e: self.min_e.max(other.min_e),
            min_n: self.min_n.max(other.min_n),
            max_e: self.max_e.min(other.max_e),
            max_n: self.max_n.min(other.max_n),
        };
        (b.min_e < b.max_e && b.min_n < b.max_n).then_some(b)
    }

    pub fn width(&self) -> f64 {
        self.max_e - self.min_e
    }

    pub fn height(&self) -> f64 {
        self.max_n - self.min_n
    }
}

/// EPSG code of ETRS89 / UTM zone 33N.
pub const EPSG_UTM33: u32 = 25833;

/// GRS80 semi-major axis in metres.
const GRS80_A: f64 = 6_378_137.0;
/// GRS80 flattening.
const GRS80_F: f64 = 1.0 / 298.257_222_101;
/// UTM scale factor on the central meridian.
const UTM_K0: f64 = 0.9996;
/// Central meridian of UTM zone 33 in degrees.
const UTM33_LON0: f64 = 15.0;
const UTM_FALSE_EASTING: f64 = 500_000.0;

/// Projects a geographic position to EPSG:25833.
pub fn utm33_forward(p: LatLon) -> ProjectedPoint {
    transverse_mercator(p).point
}

/// Geographic position of an EPSG:25833 point: Newton iteration on [`utm33_forward`], so both
/// directions agree to well below a millimetre.
pub fn utm33_inverse(p: ProjectedPoint) -> LatLon {
    /// Finite-difference step in degrees (about 0.1 m).
    const STEP: f64 = 1e-6;
    let lat = p.northing / 111_132.0;
    let mut q = LatLon {
        lat,
        lon: UTM33_LON0 + (p.easting - UTM_FALSE_EASTING) / (111_320.0 * lat.to_radians().cos()),
    };
    for _ in 0..10 {
        let f = utm33_forward(q);
        let (de, dn) = (p.easting - f.easting, p.northing - f.northing);
        if de.abs() < 1e-6 && dn.abs() < 1e-6 {
            break;
        }
        let at_lat = utm33_forward(LatLon {
            lat: q.lat + STEP,
            ..q
        });
        let at_lon = utm33_forward(LatLon {
            lon: q.lon + STEP,
            ..q
        });
        let (e_lat, e_lon) = (
            (at_lat.easting - f.easting) / STEP,
            (at_lon.easting - f.easting) / STEP,
        );
        let (n_lat, n_lon) = (
            (at_lat.northing - f.northing) / STEP,
            (at_lon.northing - f.northing) / STEP,
        );
        let det = e_lat * n_lon - e_lon * n_lat;
        q.lat += (n_lon * de - e_lon * dn) / det;
        q.lon += (e_lat * dn - n_lat * de) / det;
    }
    q
}

/// Grid convergence in degrees: angle from true north to grid north, negative west of the
/// central meridian (same sign convention as OOM).
pub fn grid_convergence_deg(p: LatLon) -> f64 {
    transverse_mercator(p).convergence_deg
}

/// Point scale factor of the projection at `p`.
pub fn point_scale_factor(p: LatLon) -> f64 {
    transverse_mercator(p).scale_factor
}

struct TransverseMercator {
    point: ProjectedPoint,
    convergence_deg: f64,
    scale_factor: f64,
}

/// Krüger series of order n^6 (Karney 2011, "Transverse Mercator with an accuracy of a few
/// nanometers").
fn transverse_mercator(p: LatLon) -> TransverseMercator {
    let f = GRS80_F;
    let e = (f * (2.0 - f)).sqrt();
    let n = f / (2.0 - f);
    let (n2, n3, n4, n5, n6) = (n * n, n.powi(3), n.powi(4), n.powi(5), n.powi(6));
    let rectifying_radius = GRS80_A / (1.0 + n) * (1.0 + n2 / 4.0 + n4 / 64.0 + n6 / 256.0);
    let alpha = [
        n / 2.0 - 2.0 * n2 / 3.0 + 5.0 * n3 / 16.0 + 41.0 * n4 / 180.0 - 127.0 * n5 / 288.0
            + 7891.0 * n6 / 37800.0,
        13.0 * n2 / 48.0 - 3.0 * n3 / 5.0 + 557.0 * n4 / 1440.0 + 281.0 * n5 / 630.0
            - 1_983_433.0 * n6 / 1_935_360.0,
        61.0 * n3 / 240.0 - 103.0 * n4 / 140.0
            + 15061.0 * n5 / 26880.0
            + 167_603.0 * n6 / 181_440.0,
        49561.0 * n4 / 161_280.0 - 179.0 * n5 / 168.0 + 6_601_661.0 * n6 / 7_257_600.0,
        34729.0 * n5 / 80640.0 - 3_418_889.0 * n6 / 1_995_840.0,
        212_378_941.0 * n6 / 319_334_400.0,
    ];

    let phi = p.lat.to_radians();
    let lambda = (p.lon - UTM33_LON0).to_radians();
    let tau = phi.tan();
    let sigma = (e * (e * phi.sin()).atanh()).sinh();
    let tau_conformal = tau * (1.0 + sigma * sigma).sqrt() - sigma * (1.0 + tau * tau).sqrt();
    let xi_prime = tau_conformal.atan2(lambda.cos());
    let eta_prime =
        (lambda.sin() / (tau_conformal * tau_conformal + lambda.cos().powi(2)).sqrt()).asinh();

    let (mut xi, mut eta, mut p_sum, mut q_sum) = (xi_prime, eta_prime, 1.0, 0.0);
    for (j, a) in alpha.iter().enumerate() {
        let k = 2.0 * (j as f64 + 1.0);
        xi += a * (k * xi_prime).sin() * (k * eta_prime).cosh();
        eta += a * (k * xi_prime).cos() * (k * eta_prime).sinh();
        p_sum += k * a * (k * xi_prime).cos() * (k * eta_prime).cosh();
        q_sum += k * a * (k * xi_prime).sin() * (k * eta_prime).sinh();
    }

    let gamma = (tau_conformal * lambda.tan() / (1.0 + tau_conformal * tau_conformal).sqrt())
        .atan()
        + q_sum.atan2(p_sum);
    let scale_factor = UTM_K0 * rectifying_radius / GRS80_A
        * (1.0 + ((1.0 - e * e) * tau * tau)).sqrt()
        * (p_sum * p_sum + q_sum * q_sum).sqrt()
        / (tau_conformal * tau_conformal + lambda.cos().powi(2)).sqrt();

    TransverseMercator {
        point: ProjectedPoint {
            easting: UTM_FALSE_EASTING + UTM_K0 * rectifying_radius * eta,
            northing: UTM_K0 * rectifying_radius * xi,
        },
        convergence_deg: gamma.to_degrees(),
        scale_factor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utm33_roundtrip() {
        for (lat, lon) in [(52.5388, 13.8433), (51.4, 11.9), (53.5, 14.7), (52.0, 15.0)] {
            let p = LatLon { lat, lon };
            let back = utm33_inverse(utm33_forward(p));
            let again = utm33_forward(back);
            let there = utm33_forward(p);
            assert!(
                (again.easting - there.easting).abs() < 0.001
                    && (again.northing - there.northing).abs() < 0.001,
                "{p:?} -> {back:?}"
            );
            assert!((back.lat - lat).abs() < 1e-8 && (back.lon - lon).abs() < 1e-8);
        }
    }

    #[test]
    fn bbox_buffer_snap_intersection() {
        let b = BoundingBox {
            min_e: 0.3,
            min_n: 0.3,
            max_e: 9.7,
            max_n: 9.7,
        };
        let buffered = b.buffered(1.0);
        assert!((buffered.min_e + 0.7).abs() < 1e-9 && (buffered.max_n - 10.7).abs() < 1e-9);
        assert_eq!(
            buffered.snapped_outward(1.0),
            BoundingBox {
                min_e: -1.0,
                min_n: -1.0,
                max_e: 11.0,
                max_n: 11.0
            }
        );
        assert!((b.width() - 9.4).abs() < 1e-9 && (b.height() - 9.4).abs() < 1e-9);
        let other = BoundingBox {
            min_e: 20.0,
            min_n: 0.0,
            max_e: 30.0,
            max_n: 10.0,
        };
        assert_eq!(b.intersection(&other), None);
        let overlap = BoundingBox {
            min_e: 5.0,
            ..other
        };
        assert_eq!(
            b.intersection(&overlap),
            Some(BoundingBox {
                min_e: 5.0,
                min_n: 0.3,
                max_e: 9.7,
                max_n: 9.7
            })
        );
    }

    /// (lat, lon, easting, northing, convergence_deg, scale factor), reference values from the
    /// Krüger series of order n^6.
    const REFERENCE: [(f64, f64, f64, f64, f64, f64); 3] = [
        (
            52.54024494927058,
            13.844335699781933,
            421622.9917,
            5821756.0323,
            -0.917390,
            0.99967540,
        ),
        (52.0, 15.0, 500000.0, 5761038.2125, 0.0, 0.99960000),
        (53.0, 12.0, 298694.2000, 5876481.1399, -2.396705, 1.00009740),
    ];

    #[test]
    fn forward_reference_points() {
        for (lat, lon, e, n, _, _) in REFERENCE {
            let p = utm33_forward(LatLon { lat, lon });
            assert!(
                (p.easting - e).abs() < 0.001,
                "easting {} vs {e}",
                p.easting
            );
            assert!(
                (p.northing - n).abs() < 0.001,
                "northing {} vs {n}",
                p.northing
            );
        }
    }

    #[test]
    fn convergence_reference_points() {
        for (lat, lon, _, _, gamma, _) in REFERENCE {
            let g = grid_convergence_deg(LatLon { lat, lon });
            assert!((g - gamma).abs() < 1e-6, "convergence {g} vs {gamma}");
        }
    }

    #[test]
    fn scale_factor_reference_points() {
        for (lat, lon, _, _, _, k) in REFERENCE {
            let s = point_scale_factor(LatLon { lat, lon });
            assert!((s - k).abs() < 1e-8, "scale factor {s} vs {k}");
        }
    }
}
