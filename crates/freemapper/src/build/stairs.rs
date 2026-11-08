//! ALKIS stairs (Berlin `Treppe`, `Freitreppe` areas) → 532.6–532.9 candidates (Berlin plus 1).
//!
//! A stair polygon becomes one line along its walking direction. The oriented minimum rectangle
//! of the polygon gives two axes; the walking direction is the axis whose ends differ more in
//! height in the terrain model (DGM1, sampled in the map's CRS), and the line runs uphill. If
//! neither axis rises [`STAIR_MIN_RISE_M`], or no terrain model is stored, the long axis is used
//! and the candidate is tagged `freemapper:direction=guessed`. The width across the walking
//! direction, in metres on the ground, picks the footprint symbol.

use chrono::NaiveDate;
use geo::{Distance as _, Euclidean, MinimumRotatedRect as _};
use geo_types::{Coord, Polygon};
use serde_json::Value;

use crate::build::berlin::{feature_polygons, provenance_tags};
use crate::crs::{Crs, ProjectedPoint};
use crate::derive::ndsm::bilinear;
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapObject, MapObjectKind, SymbolSet};
use crate::raster::Raster;

/// The layer the stair areas come from.
const STAIR_LAYER: &str = "bauwerke-flaechen";
/// `bezbwf` of the stair areas.
const STAIR_KINDS: [&str; 2] = ["Treppe", "Freitreppe"];
/// An axis whose ends differ less than this in height does not decide the walking direction.
pub const STAIR_MIN_RISE_M: f64 = 0.3;
/// Upper width bounds (m, exclusive) of the footprint symbols; wider stairs get 532.9 (4 m).
const FOOTPRINTS: [(f64, &str); 3] = [(1.8, "532.6"), (2.5, "532.7"), (3.5, "532.8")];
const WIDEST_FOOTPRINT: &str = "532.9";

/// The ISSprOM footprint symbol (532.6 = 1.6 m … 532.9 = 4 m) for a stair `width_m` wide on the
/// ground.
pub fn footprint_code(width_m: f64) -> &'static str {
    FOOTPRINTS
        .iter()
        .find(|(below, _)| width_m < *below)
        .map_or(WIDEST_FOOTPRINT, |(_, code)| code)
}

/// One axis of a stair rectangle: the centre line from side to side, and the width across it.
struct Axis {
    from: Coord<f64>,
    to: Coord<f64>,
    width: f64,
}

impl Axis {
    fn length(&self) -> f64 {
        (self.to - self.from).x.hypot((self.to - self.from).y)
    }

    /// The height difference `to` minus `from` in `terrain`, if both ends have a height.
    fn rise(&self, terrain: &Raster<f32>) -> Option<f64> {
        let height = |c: Coord<f64>| bilinear(terrain, c.x, c.y).map(f64::from);
        Some(height(self.to)? - height(self.from)?)
    }

    fn reversed(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
            width: self.width,
        }
    }
}

/// The two axes of the oriented minimum rectangle of `polygon`, the long one first; none for a
/// degenerate polygon.
fn axes(polygon: &Polygon<f64>) -> Option<[Axis; 2]> {
    let rect = polygon.minimum_rotated_rect()?;
    let c: Vec<Coord<f64>> = rect.exterior().coords().copied().collect();
    let [p0, p1, p2, p3, ..] = c[..] else {
        return None;
    };
    let mid = |a: Coord<f64>, b: Coord<f64>| (a + b) / 2.0;
    let side = |a: Coord<f64>, b: Coord<f64>| (b - a).x.hypot((b - a).y);
    // Along p0 → p1, across p1 → p2, and the other way round.
    let first = Axis {
        from: mid(p0, p3),
        to: mid(p1, p2),
        width: side(p1, p2),
    };
    let second = Axis {
        from: mid(p0, p1),
        to: mid(p3, p2),
        width: side(p0, p1),
    };
    if first.length() <= 0.0 || second.length() <= 0.0 {
        return None;
    }
    Some(if first.length() >= second.length() {
        [first, second]
    } else {
        [second, first]
    })
}

/// The walking axis of a stair, uphill, and whether it is only the long axis (no terrain model,
/// or flatter than [`STAIR_MIN_RISE_M`]).
fn walking_axis([long, short]: [Axis; 2], terrain: Option<&Raster<f32>>) -> (Axis, bool) {
    let rises = terrain.and_then(|t| Some((long.rise(t)?, short.rise(t)?)));
    match rises {
        Some((a, b)) if a.abs().max(b.abs()) >= STAIR_MIN_RISE_M => {
            let (axis, rise) = if a.abs() >= b.abs() {
                (long, a)
            } else {
                (short, b)
            };
            (if rise < 0.0 { axis.reversed() } else { axis }, false)
        }
        _ => (long, true),
    }
}

/// 532.6–532.9 candidates for the stair areas among the `bauwerke-flaechen` features within
/// `buffer_m` of `map_area` (map CRS). `terrain` is the DGM1 in the map's CRS, if one is stored;
/// `currency` is the fetch date of the layer.
pub fn stair_objects(
    features: &[Value],
    terrain: Option<&Raster<f32>>,
    map_area: &Polygon<f64>,
    buffer_m: f64,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
    currency: NaiveDate,
) -> Result<Vec<MapObject>, GeorefError> {
    let crs = Crs::from_epsg(georef.epsg).unwrap_or(Crs::Utm33);
    let to_map = |c: Coord<f64>| {
        georef.projected_to_map(ProjectedPoint {
            easting: c.x,
            northing: c.y,
        })
    };
    let mut objects = Vec::new();
    for feature in features.iter().filter(|f| {
        f["properties"]["bezbwf"]
            .as_str()
            .is_some_and(|kind| STAIR_KINDS.contains(&kind))
    }) {
        for part in feature_polygons(feature, crs) {
            if Euclidean.distance(&part, map_area) > buffer_m {
                continue;
            }
            let Some(axes) = axes(&part) else {
                continue;
            };
            let (axis, guessed) = walking_axis(axes, terrain);
            let symbol_id = symbol_set
                .id_for_code(footprint_code(axis.width))
                .expect("symbol set defines 532.6-532.9");
            let mut tags = provenance_tags(STAIR_LAYER, feature, currency);
            if guessed {
                tags.push(("freemapper:direction".to_string(), "guessed".to_string()));
            }
            objects.push(MapObject {
                symbol_id,
                kind: MapObjectKind::Path,
                coords: vec![to_map(axis.from)?, to_map(axis.to)?],
                tags,
            });
        }
    }
    Ok(objects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::MapKind;
    use crate::raster::GridSpec;

    const E: f64 = 392_400.0;
    const N: f64 = 5_820_400.0;

    fn setup(kind: MapKind) -> (Georeferencing, Polygon<f64>, SymbolSet) {
        let centre = Crs::Utm33
            .inverse(ProjectedPoint {
                easting: E,
                northing: N,
            })
            .unwrap();
        let georef =
            Georeferencing::new(centre, kind.default_scale(), "2026-11-08".parse().unwrap())
                .unwrap();
        let (e, n) = (georef.ref_projected.easting, georef.ref_projected.northing);
        let ring = vec![
            (e - 100.0, n - 100.0),
            (e + 100.0, n - 100.0),
            (e + 100.0, n + 100.0),
            (e - 100.0, n + 100.0),
            (e - 100.0, n - 100.0),
        ];
        (georef, Polygon::new(ring.into(), vec![]), kind.symbol_set())
    }

    /// A 1 m DGM around the reference point with height `h(e, n)` (metres from the reference
    /// point) at each cell centre.
    fn terrain(h: impl Fn(f64, f64) -> f32) -> Raster<f32> {
        let grid = GridSpec {
            west: E - 150.0,
            north: N + 150.0,
            pixel_size: 1.0,
            width: 300,
            height: 300,
        };
        let mut raster = Raster::filled(grid, f32::NAN);
        for y in 0..grid.height {
            for x in 0..grid.width {
                let e = grid.west + f64::from(x) + 0.5 - E;
                let n = grid.north - f64::from(y) - 0.5 - N;
                raster.set(x, y, h(e, n));
            }
        }
        raster
    }

    /// A `Treppe` feature with the corners `points` (metres from the reference point).
    fn stair(kind: &str, points: &[(f64, f64)]) -> Value {
        let mut ring: Vec<[f64; 2]> = points.iter().map(|(e, n)| [E + e, N + n]).collect();
        ring.push(ring[0]);
        serde_json::json!({
            "type": "Feature", "id": "bauwerkeflaechen.DEBE00YYUK0000Ah",
            "geometry": {"type": "MultiPolygon", "coordinates": [[ring]]},
            "properties": {"uuid": "DEBE00YYUK0000Ah", "bwf": "1630", "bezbwf": kind}
        })
    }

    /// A `w` (east) × `h` (north) m stair with its south-west corner at the reference point.
    fn rect_stair(w: f64, h: f64) -> Value {
        stair("Treppe", &[(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)])
    }

    fn objects(kind: MapKind, features: &[Value], dgm: Option<&Raster<f32>>) -> Vec<MapObject> {
        let (georef, area, set) = setup(kind);
        stair_objects(
            features,
            dgm,
            &area,
            20.0,
            &georef,
            &set,
            "2026-11-09".parse().unwrap(),
        )
        .unwrap()
    }

    fn code(kind: MapKind, object: &MapObject) -> String {
        let set = kind.symbol_set();
        ["532.6", "532.7", "532.8", "532.9"]
            .into_iter()
            .find(|c| set.id_for_code(c) == Some(object.symbol_id))
            .expect("a stair symbol")
            .to_string()
    }

    /// The line ends in metres from the reference point.
    fn ends(kind: MapKind, object: &MapObject) -> [(f64, f64); 2] {
        let (georef, _, _) = setup(kind);
        let p = |i: usize| {
            let p = georef.map_to_projected(object.coords[i]);
            (p.easting - E, p.northing - N)
        };
        [p(0), p(1)]
    }

    fn close(a: (f64, f64), b: (f64, f64)) -> bool {
        (a.0 - b.0).abs() < 0.05 && (a.1 - b.1).abs() < 0.05
    }

    fn direction(object: &MapObject) -> Option<&str> {
        object
            .tags
            .iter()
            .find(|(k, _)| k == "freemapper:direction")
            .map(|(_, v)| v.as_str())
    }

    #[test]
    fn stair_direction_follows_the_dtm() {
        let kind = MapKind::Sprint;
        // 6 m east × 3 m north, the ground rising 0.2 m per metre to the north: the short axis
        // rises 0.6 m, the long one not at all. The steps go north, 6 m wide.
        let north = terrain(|_, n| 30.0 + 0.2 * n as f32);
        let found = objects(kind, &[rect_stair(6.0, 3.0)], Some(&north));
        assert_eq!(found.len(), 1);
        let [from, to] = ends(kind, &found[0]);
        assert!(
            close(from, (3.0, 0.0)) && close(to, (3.0, 3.0)),
            "{from:?} → {to:?}"
        );
        assert_eq!(code(kind, &found[0]), "532.9");
        assert_eq!(direction(&found[0]), None);
        let tags: Vec<(&str, &str)> = found[0]
            .tags
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(
            tags,
            [
                ("freemapper:source", "geoportal-berlin/bauwerke-flaechen"),
                ("freemapper:source_id", "DEBE00YYUK0000Ah"),
                ("freemapper:currency", "2026-11-09"),
                ("freemapper:alkis", "Treppe"),
            ]
        );

        // The ground falls to the east: the long axis, uphill westwards, 3 m wide.
        let west = terrain(|e, _| 30.0 - 0.1 * e as f32);
        let found = objects(kind, &[rect_stair(6.0, 3.0)], Some(&west));
        let [from, to] = ends(kind, &found[0]);
        assert!(
            close(from, (6.0, 1.5)) && close(to, (0.0, 1.5)),
            "{from:?} → {to:?}"
        );
        assert_eq!(code(kind, &found[0]), "532.8");

        // A Freitreppe shaped like ALKIS DEBE00YYUK0000Aj (a parallelogram, 15 m long), on a
        // slope to the north: its long axis uphill, 3 m wide across the rectangle.
        let freitreppe = stair(
            "Freitreppe",
            &[
                (9.981, -17.345),
                (6.849, -15.896),
                (8.529, -30.627),
                (11.661, -32.076),
            ],
        );
        let found = objects(kind, &[freitreppe], Some(&north));
        let [from, to] = ends(kind, &found[0]);
        assert!(to.1 - from.1 > 14.0, "{from:?} → {to:?}");
        assert_eq!(code(kind, &found[0]), "532.8");
        assert_eq!(direction(&found[0]), None);
    }

    #[test]
    fn flat_stair_uses_long_axis_and_is_tagged() {
        let kind = MapKind::Schoolyard;
        // 0.25 m over 3 m: below the 0.3 m rise.
        let gentle = terrain(|_, n| 30.0 + 0.08 * n as f32);
        let no_data = terrain(|_, _| f32::NAN);
        for dgm in [Some(&gentle), Some(&no_data), None] {
            let found = objects(kind, &[rect_stair(6.0, 3.0)], dgm);
            assert_eq!(found.len(), 1);
            let [from, to] = ends(kind, &found[0]);
            assert!((from.1 - 1.5).abs() < 0.05 && (to.1 - 1.5).abs() < 0.05);
            assert!(
                ((to.0 - from.0).abs() - 6.0).abs() < 0.05,
                "{from:?} → {to:?}"
            );
            assert_eq!(direction(&found[0]), Some("guessed"));
            assert_eq!(code(kind, &found[0]), "532.8");
        }
    }

    #[test]
    fn stair_width_picks_footprint_symbol() {
        for (width, expected) in [
            (1.0, "532.6"),
            (1.79, "532.6"),
            (1.8, "532.7"),
            (2.49, "532.7"),
            (2.5, "532.8"),
            (3.49, "532.8"),
            (3.5, "532.9"),
            (12.0, "532.9"),
        ] {
            assert_eq!(footprint_code(width), expected, "{width} m");
        }
        // Metres on the ground on both map kinds, not millimetres on paper.
        for kind in [MapKind::Sprint, MapKind::Schoolyard] {
            let found = objects(kind, &[rect_stair(1.5, 8.0)], None);
            assert_eq!(code(kind, &found[0]), "532.6", "{kind:?}");
            let found = objects(kind, &[rect_stair(8.0, 2.0)], None);
            assert_eq!(code(kind, &found[0]), "532.7", "{kind:?}");
        }
        // Other areas are no stairs.
        let mut wall = rect_stair(8.0, 2.0);
        wall["properties"]["bezbwf"] = "Mauer".into();
        assert!(objects(MapKind::Sprint, &[wall], None).is_empty());
    }
}
