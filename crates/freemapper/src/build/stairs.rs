//! ALKIS stairs (Berlin `Treppe`, `Freitreppe` areas) → 532.6–532.9 candidates (Berlin plus 1).
//!
//! A stair polygon becomes one line along its walking direction. The oriented minimum rectangle
//! of the polygon gives two axes; the walking direction is the axis whose ends differ more in
//! height in the terrain model (DGM1, sampled in the map's CRS), and the line runs uphill. If
//! neither axis rises [`STAIR_MIN_RISE_M`], or no terrain model is stored, the long axis is used
//! and the candidate is tagged `freemapper:direction=guessed`; a stair at a building façade
//! (entrance steps) then faces the building instead. The width across the walking direction,
//! as paper width at the map scale, picks the footprint symbol nearest to it.

use chrono::NaiveDate;
use geo::{Distance as _, Euclidean, MinimumRotatedRect as _};
use geo_types::{Coord, Polygon};
use serde_json::Value;

use crate::build::berlin::{feature_polygons, provenance_tags};
use crate::crs::{Crs, ProjectedPoint};
use crate::derive::ndsm::bilinear;
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapObject, MapObjectKind, SymbolSet};
use crate::project::MapKind;
use crate::raster::Raster;

/// The layer the stair areas come from.
const STAIR_LAYER: &str = "bauwerke-flaechen";
/// `bezbwf` of the stair areas.
const STAIR_KINDS: [&str; 2] = ["Treppe", "Freitreppe"];
/// An axis whose ends differ less than this in height does not decide the walking direction.
pub const STAIR_MIN_RISE_M: f64 = 0.3;
/// The footprint symbols are 0.4, 0.5, 0.75 and 1 mm wide on paper (symbols at 100 %; at
/// 1:4000 the 1.6 / 2 / 3 / 4 m of their names). Upper bounds (mm, exclusive) of each: the
/// midpoints to the next one; wider stairs get 532.9.
const FOOTPRINTS: [(f64, &str); 3] = [(0.45, "532.6"), (0.625, "532.7"), (0.875, "532.8")];
const WIDEST_FOOTPRINT: &str = "532.9";
/// A flat stair whose side lies this close to a building is taken for entrance steps.
const FACADE_DISTANCE_M: f64 = 0.5;

/// The ISSprOM footprint symbol (532.6 … 532.9) nearest to a stair `paper_mm` wide on paper,
/// measured at 100 % symbol size (the ground width over metres per mm and the enlargement).
pub fn footprint_code(paper_mm: f64) -> &'static str {
    FOOTPRINTS
        .iter()
        .find(|(below, _)| paper_mm < *below)
        .map_or(WIDEST_FOOTPRINT, |(_, code)| code)
}

fn is_stair(feature: &Value) -> bool {
    feature["properties"]["bezbwf"]
        .as_str()
        .is_some_and(|kind| STAIR_KINDS.contains(&kind))
}

/// Whether any of the `features` of a layer is a stair area (`derive` keeps the terrain then).
pub fn has_stairs(features: &[Value]) -> bool {
    features.iter().any(is_stair)
}

/// What the stairs are built from.
pub struct StairInput<'a> {
    /// The `bauwerke-flaechen` features (EPSG:25833); only the stair areas are used.
    pub features: &'a [Value],
    /// The DGM1 in the map's CRS, if one is stored.
    pub terrain: Option<&'a Raster<f32>>,
    /// The 521 footprints in the map's CRS, for the direction of flat entrance steps.
    pub buildings: &'a [Polygon<f64>],
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

/// The walking axis of a stair, uphill, and whether it is only guessed (no terrain model, or
/// flatter than [`STAIR_MIN_RISE_M`]): then towards a building the stair lies at
/// ([`FACADE_DISTANCE_M`]), else the long axis.
fn walking_axis(
    [long, short]: [Axis; 2],
    terrain: Option<&Raster<f32>>,
    buildings: &[Polygon<f64>],
) -> (Axis, bool) {
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
        _ => (facing_building([long, short], buildings), true),
    }
}

/// Of the four axis ends, the one nearest to a building within [`FACADE_DISTANCE_M`] gives the
/// walking axis, towards the building; without one, the long (first) axis.
fn facing_building([long, short]: [Axis; 2], buildings: &[Polygon<f64>]) -> Axis {
    let ends = geo_types::MultiPoint::from(vec![long.from, long.to, short.from, short.to]);
    let near: Vec<&Polygon<f64>> = buildings
        .iter()
        .filter(|b| Euclidean.distance(&ends, *b) <= FACADE_DISTANCE_M)
        .collect();
    let distance = |c: Coord<f64>| {
        near.iter()
            .map(|b| Euclidean.distance(&geo_types::Point(c), *b))
            .fold(f64::INFINITY, f64::min)
    };
    let ends = [
        (distance(long.to), 0, false),
        (distance(long.from), 0, true),
        (distance(short.to), 1, false),
        (distance(short.from), 1, true),
    ];
    let nearest = ends
        .into_iter()
        .filter(|(d, _, _)| *d <= FACADE_DISTANCE_M)
        .min_by(|a, b| a.0.total_cmp(&b.0));
    match nearest {
        Some((_, 1, reverse)) => {
            if reverse {
                short.reversed()
            } else {
                short
            }
        }
        Some((_, _, true)) => long.reversed(),
        _ => long,
    }
}

/// 532.6–532.9 candidates for the stair areas of `input` within `buffer_m` of `map_area` (map
/// CRS), with the footprints of the symbols of `kind`; `currency` is the fetch date of the
/// layer.
pub fn stair_objects(
    input: &StairInput<'_>,
    kind: MapKind,
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
    // Ground metres per paper millimetre at 100 % symbol size.
    let metres_per_mm = f64::from(georef.scale_denominator) / 1000.0 * kind.symbol_enlargement();
    let mut objects = Vec::new();
    for feature in input.features.iter().filter(|f| is_stair(f)) {
        for part in feature_polygons(feature, crs) {
            if Euclidean.distance(&part, map_area) > buffer_m {
                continue;
            }
            let Some(axes) = axes(&part) else {
                continue;
            };
            let (axis, guessed) = walking_axis(axes, input.terrain, input.buildings);
            let symbol_id = symbol_set
                .id_for_code(footprint_code(axis.width / metres_per_mm))
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
        objects_beside(kind, features, dgm, &[])
    }

    fn objects_beside(
        kind: MapKind,
        features: &[Value],
        terrain: Option<&Raster<f32>>,
        buildings: &[Polygon<f64>],
    ) -> Vec<MapObject> {
        let (georef, area, set) = setup(kind);
        stair_objects(
            &StairInput {
                features,
                terrain,
                buildings,
            },
            kind,
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
            // 3 m wide at 1:1000 × 1.25: wider than the widest footprint (1.25 m).
            assert_eq!(code(kind, &found[0]), "532.9");
        }
    }

    #[test]
    fn flat_stair_at_a_building_faces_it() {
        let kind = MapKind::Schoolyard;
        let flat = terrain(|_, _| 30.0);
        // Entrance steps along a façade: 8 m east × 2 m north, the building north of them.
        let rect = |e: f64, n: f64, w: f64, h: f64| -> Polygon<f64> {
            let ring = vec![(e, n), (e + w, n), (e + w, n + h), (e, n + h), (e, n)];
            Polygon::new(ring.into(), vec![])
        };
        let building = rect(E - 5.0, N + 2.0, 20.0, 10.0);
        let found = objects_beside(kind, &[rect_stair(8.0, 2.0)], Some(&flat), &[building]);
        let [from, to] = ends(kind, &found[0]);
        assert!(
            close(from, (4.0, 0.0)) && close(to, (4.0, 2.0)),
            "{from:?} → {to:?}"
        );
        assert_eq!(direction(&found[0]), Some("guessed"));
        // A building 5 m away is no façade of the stair: the long axis.
        let away = rect(E - 5.0, N + 7.0, 20.0, 10.0);
        let found = objects_beside(kind, &[rect_stair(8.0, 2.0)], Some(&flat), &[away]);
        let [from, to] = ends(kind, &found[0]);
        assert!(
            ((to.0 - from.0).abs() - 8.0).abs() < 0.05,
            "{from:?} → {to:?}"
        );
        assert_eq!(direction(&found[0]), Some("guessed"));
        // A rising DGM still decides over the building.
        let east = terrain(|e, _| 30.0 + 0.1 * e as f32);
        let building = rect(E - 5.0, N + 2.0, 20.0, 10.0);
        let found = objects_beside(kind, &[rect_stair(8.0, 2.0)], Some(&east), &[building]);
        let [from, to] = ends(kind, &found[0]);
        assert!(
            close(from, (0.0, 1.0)) && close(to, (8.0, 1.0)),
            "{from:?} → {to:?}"
        );
        assert_eq!(direction(&found[0]), None);
    }

    #[test]
    fn stair_width_picks_footprint_symbol() {
        // Paper widths of 532.6-532.9 at 100 %: 0.4, 0.5, 0.75 and 1 mm; the nearest one wins.
        for (paper_mm, expected) in [
            (0.3, "532.6"),
            (0.449, "532.6"),
            (0.45, "532.7"),
            (0.62, "532.7"),
            (0.625, "532.8"),
            (0.87, "532.8"),
            (0.875, "532.9"),
            (3.0, "532.9"),
        ] {
            assert_eq!(footprint_code(paper_mm), expected, "{paper_mm} mm");
        }
        // Sprint 1:4000: the names of the symbols (1.6 / 2 / 3 / 4 m on the ground).
        for (width, expected) in [
            (1.5, "532.6"),
            (2.0, "532.7"),
            (3.0, "532.8"),
            (4.0, "532.9"),
        ] {
            let found = objects(MapKind::Sprint, &[rect_stair(width, 8.0)], None);
            assert_eq!(code(MapKind::Sprint, &found[0]), expected, "{width} m");
        }
        // Schoolyard 1:1000 with symbols at 125 %: 1.25 m per (100 %) mm, so most stairs are
        // drawn with the widest footprint, the one closest to their real width.
        for (width, expected) in [
            (0.5, "532.6"),
            (0.7, "532.7"),
            (1.5, "532.9"),
            (3.0, "532.9"),
        ] {
            let found = objects(MapKind::Schoolyard, &[rect_stair(width, 8.0)], None);
            assert_eq!(code(MapKind::Schoolyard, &found[0]), expected, "{width} m");
        }
        // Other areas are no stairs.
        let mut wall = rect_stair(8.0, 2.0);
        wall["properties"]["bezbwf"] = "Mauer".into();
        assert!(objects(MapKind::Sprint, &[wall], None).is_empty());
    }
}
