//! Cliffs on the forest map (ISOM 2017-2, 201 and 206; ADR 0125).
//!
//! OpenStreetMap draws rock country cliff by cliff: at the Bastei 2144 lines, up to 127 on one
//! square centimetre of paper, often several along one face. The norm keeps 0.25 mm between two
//! impassable cliffs, draws none shorter than 0.6 mm and shows a rock pillar or a massive cliff in
//! plan shape (206). After the content settings placed the objects, the cliffs on the map are
//! generalized:
//!
//! * A cliff whose terrain drops less than [`MIN_DROP_M`] across it is no impassable cliff at
//!   this scale; it goes to the candidates.
//! * A closed cliff around a rock pillar of the norm's size becomes the area 206.
//! * The other cliffs keep the norm's gap, the highest and longest first; a cliff too close to one
//!   kept or to a pillar, or too short, goes to the candidates.
//!
//! The cliffs that go are hidden in the training map ([`HIDDEN_KEY`]` = generalized`), so it shows
//! the map's choice, and name why in `freemapper:generalized`. The tall cliffs, kept or not, and
//! the pillars are returned: the contours stop at them, so they do not pile up in a rock face.

use geo::Simplify as _;
use geo::{Area as _, BoundingRect as _, Buffer as _, Distance as _, Euclidean, Length as _};
use geo_types::{Coord, LineString, Polygon, Rect};

use crate::build::points::HIDDEN_KEY;
use crate::build::symbol_measures::{CLIFF_MM, CLIFF_TICK_MM};
use crate::crs::ProjectedPoint;
use crate::derive::ndsm::bilinear;
use crate::georef::Georeferencing;
use crate::measure::{PaperMm, SymbolMm, SymbolMm2, SymbolScale};
use crate::omap::coord::FLAG_CLOSE;
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet, closed_ring};
use crate::project::MapKind;
use crate::raster::Raster;

/// The impassable cliff and the gigantic boulder or rock pillar, in plan shape.
const CLIFF_CODE: &str = "201";
const PILLAR_CODE: &str = "206";
/// Ruling: a cliff whose terrain drops less across it is no impassable cliff at forest scale.
pub const MIN_DROP_M: f64 = 3.0;
/// ISOM 2017-2, 201: the gap to another impassable cliff and the minimum length.
const CLIFF_GAP_MM: SymbolMm = SymbolMm(0.25);
const CLIFF_MIN_LENGTH_MM: SymbolMm = SymbolMm(0.6);
/// ISOM 2017-2, 206: the gap to other impassable features, the minimum width and area.
const PILLAR_GAP_MM: SymbolMm = SymbolMm(0.15);
const PILLAR_MIN_WIDTH_MM: SymbolMm = SymbolMm(0.25);
const PILLAR_MIN_AREA_MM2: SymbolMm2 = SymbolMm2(0.3);
/// Ruling: a closed cliff around more is a plateau or a massif, no pillar (about 47 m across at
/// 1:15 000).
const PILLAR_MAX_AREA_MM2: SymbolMm2 = SymbolMm2(10.0);
/// The lines are smoothed: OSM traces every niche of a rock face, and a bend shorter than about
/// the distance of two tags (0.5 mm) reads as zigzag. Gaussian along the line with this sigma
/// (ruling, from a comparison on the Bastei); then simplified like the contours.
const SMOOTH_SIGMA_MM: SymbolMm = SymbolMm(0.3);
const SMOOTH_STEP_M: f64 = 1.0;
const SIMPLIFY_MM: PaperMm = PaperMm(0.1);
/// How far from the cliff the heights of its two sides are read, and how often (ground metres).
const DROP_PROBE_M: f64 = 2.5;
const DROP_STEP_M: f64 = 2.0;
/// The value of [`HIDDEN_KEY`] on a cliff the generalization took off the map.
pub const GENERALIZED: &str = "generalized";
const GENERALIZED_KEY: &str = "freemapper:generalized";

/// One cliff of the map, in the map's CRS.
struct Cliff {
    index: usize,
    /// The line as drawn, smoothed and simplified ...
    line: LineString<f64>,
    /// ... and only simplified: the plan shape of a pillar, which smoothing would shrink.
    outline: LineString<f64>,
    closed: bool,
    /// The median drop of the terrain across it; `None` without terrain.
    drop_m: Option<f64>,
    length_m: f64,
    frame: Rect<f64>,
}

/// Generalizes the impassable cliffs on the forest map (`map`); the ones that go are moved to
/// `candidates`. `terrain` is the height model of the project, in the map's CRS; without it every
/// cliff counts as tall. Returns where the tall cliffs and the pillars stand, for the contours.
pub fn generalize_cliffs(
    map: &mut Vec<MapObject>,
    candidates: &mut Vec<MapObject>,
    terrain: Option<&Raster<f32>>,
    map_kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Vec<Polygon<f64>> {
    let Some(cliff_id) = symbol_set.id_for_code(CLIFF_CODE) else {
        return Vec::new();
    };
    if map_kind != MapKind::Forest {
        return Vec::new();
    }
    let scale = SymbolScale::new(map_kind, georef.scale_denominator);
    let pillar_id = symbol_set.id_for_code(PILLAR_CODE);
    let tolerance = SIMPLIFY_MM.metres(georef.scale_denominator);
    let cliffs: Vec<Cliff> = map
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.symbol_id == cliff_id
                && matches!(o.kind, MapObjectKind::Path)
                && !o.tags.iter().any(|(k, _)| k == HIDDEN_KEY)
        })
        .filter_map(|(index, o)| {
            let closed = o.coords.len() > 2
                && (o.coords.first() == o.coords.last()
                    || o.coords.iter().any(|c| c.flags & FLAG_CLOSE != 0));
            let mut points: Vec<Coord<f64>> = o
                .coords
                .iter()
                .map(|&c| {
                    let p = georef.map_to_projected(c);
                    Coord {
                        x: p.easting,
                        y: p.northing,
                    }
                })
                .collect();
            if closed && points.first() != points.last() {
                points.push(points[0]);
            }
            let outline = LineString::new(points);
            let line = smooth(&outline, SMOOTH_SIGMA_MM.metres(scale), closed).simplify(tolerance);
            let frame = line.bounding_rect()?;
            Some(Cliff {
                index,
                drop_m: terrain.and_then(|t| drop_across(&line, t)),
                length_m: Euclidean.length(&line),
                closed,
                outline: outline.simplify(tolerance),
                line,
                frame,
            })
        })
        .collect();
    if cliffs.is_empty() {
        return Vec::new();
    }
    let tall = |c: &Cliff| c.drop_m.is_none_or(|d| d >= MIN_DROP_M);
    // Why each cliff left the map, by its index in `map`; the pillars that replace closed ones.
    let mut gone: Vec<Option<String>> = vec![None; map.len()];
    let mut pillars: Vec<(usize, Polygon<f64>)> = Vec::new();
    let pillar_area =
        PILLAR_MIN_AREA_MM2.square_metres(scale)..=PILLAR_MAX_AREA_MM2.square_metres(scale);
    let pillar_half_width = PILLAR_MIN_WIDTH_MM.metres(scale) / 2.0;
    for cliff in &cliffs {
        if !tall(cliff) {
            gone[cliff.index] = Some(format!("cliff lower than {MIN_DROP_M} m"));
            continue;
        }
        if !cliff.closed || pillar_id.is_none() {
            continue;
        }
        let shape = Polygon::new(cliff.outline.clone(), Vec::new());
        let area = shape.unsigned_area();
        let wide_enough = shape.buffer(-pillar_half_width).unsigned_area() > 0.0;
        if pillar_area.contains(&area) && wide_enough {
            pillars.push((cliff.index, shape));
        }
    }
    // The other tall cliffs keep the gap, the highest and longest first. Two parallel cliffs
    // need the gap between the line of one and the tags of the other.
    let conflict_m =
        (CLIFF_GAP_MM.0 + CLIFF_MM.0 / 2.0 + CLIFF_TICK_MM.0) * scale.metres_per_symbol_mm();
    let pillar_conflict_m = (PILLAR_GAP_MM.0 + CLIFF_TICK_MM.0) * scale.metres_per_symbol_mm();
    let min_length_m = CLIFF_MIN_LENGTH_MM.metres(scale);
    let is_pillar = |index: usize| pillars.iter().any(|(i, _)| *i == index);
    let mut order: Vec<&Cliff> = cliffs
        .iter()
        .filter(|c| tall(c) && !is_pillar(c.index))
        .collect();
    order.sort_by(|a, b| {
        let weight = |c: &Cliff| c.drop_m.unwrap_or(1.0) * c.length_m;
        weight(b).total_cmp(&weight(a))
    });
    let near = |a: &Rect<f64>, b: &Rect<f64>, reach: f64| {
        a.min().x - reach <= b.max().x
            && b.min().x - reach <= a.max().x
            && a.min().y - reach <= b.max().y
            && b.min().y - reach <= a.max().y
    };
    let mut kept: Vec<&Cliff> = Vec::new();
    for cliff in order {
        let reason = if cliff.length_m < min_length_m {
            Some(format!("cliff shorter than {} mm", CLIFF_MIN_LENGTH_MM.0))
        } else if pillars.iter().any(|(_, p)| {
            p.bounding_rect()
                .is_some_and(|r| near(&r, &cliff.frame, pillar_conflict_m))
                && Euclidean.distance(&cliff.line, p) < pillar_conflict_m
        }) {
            Some("cliff beside a rock pillar".to_string())
        } else if kept.iter().any(|k| {
            near(&k.frame, &cliff.frame, conflict_m)
                && Euclidean.distance(&k.line, &cliff.line) < conflict_m
        }) {
            Some(format!(
                "cliff closer than {} mm to a higher one",
                CLIFF_GAP_MM.0
            ))
        } else {
            None
        };
        match reason {
            Some(reason) => gone[cliff.index] = Some(reason),
            None => kept.push(cliff),
        }
    }
    // Where the contours stop: around every tall cliff, as far as its line and tags reach, and
    // around the pillars by their gap.
    let reach_m = (CLIFF_MM.0 / 2.0 + CLIFF_TICK_MM.0) * scale.metres_per_symbol_mm();
    let mut rock: Vec<Polygon<f64>> = cliffs
        .iter()
        .filter(|c| tall(c) && !is_pillar(c.index))
        .flat_map(|c| c.line.buffer(reach_m).0)
        .collect();
    rock.extend(
        pillars
            .iter()
            .flat_map(|(_, p)| p.buffer(PILLAR_GAP_MM.metres(scale)).0),
    );
    // The kept cliffs take their simplified line; the pillars replace their closed cliffs.
    let to_map = |line: &LineString<f64>| -> Option<Vec<(i32, i32)>> {
        line.coords()
            .map(|c| {
                georef
                    .projected_to_map(ProjectedPoint {
                        easting: c.x,
                        northing: c.y,
                    })
                    .ok()
                    .map(|m| (m.x, m.y))
            })
            .collect()
    };
    let simplified: Vec<(usize, Vec<MapCoord>)> = kept
        .iter()
        .filter_map(|c| {
            let coords = to_map(&c.line)?;
            let coords = if c.closed {
                closed_ring(&coords)
            } else {
                coords
                    .into_iter()
                    .map(|(x, y)| MapCoord::new(x, y))
                    .collect()
            };
            Some((c.index, coords))
        })
        .collect();
    for (index, coords) in simplified {
        map[index].coords = coords;
    }
    let mut added = Vec::new();
    for (index, shape) in &pillars {
        let (Some(pillar_id), Some(coords)) = (pillar_id, to_map(shape.exterior())) else {
            continue;
        };
        let mut tags = map[*index].tags.clone();
        tags.push((
            GENERALIZED_KEY.to_string(),
            "rock pillar from a closed cliff".to_string(),
        ));
        added.push(MapObject {
            symbol_id: pillar_id,
            kind: MapObjectKind::Path,
            coords: closed_ring(&coords),
            tags,
        });
        gone[*index] = Some(String::new());
    }
    let mut kept_map = Vec::with_capacity(map.len());
    for (index, mut object) in map.drain(..).enumerate() {
        match gone[index].take() {
            None => kept_map.push(object),
            // A cliff a pillar replaced goes for good: the pillar is the same rock.
            Some(reason) if reason.is_empty() => {}
            Some(reason) => {
                object
                    .tags
                    .push((HIDDEN_KEY.to_string(), GENERALIZED.to_string()));
                object.tags.push((GENERALIZED_KEY.to_string(), reason));
                candidates.push(object);
            }
        }
    }
    kept_map.extend(added);
    *map = kept_map;
    rock
}

/// `line` resampled every [`SMOOTH_STEP_M`] and smoothed with a Gaussian of `sigma_m` along it.
/// An open line keeps its end points (the window shrinks towards them); a closed one wraps round.
fn smooth(line: &LineString<f64>, sigma_m: f64, closed: bool) -> LineString<f64> {
    let length = Euclidean.length(line);
    if sigma_m <= 0.0 || length < 2.0 * SMOOTH_STEP_M {
        return line.clone();
    }
    let count = (length / SMOOTH_STEP_M).round().max(2.0) as usize;
    let samples: Vec<Coord<f64>> = (0..=count)
        .filter_map(|k| {
            use geo::InterpolatableLine as _;
            line.point_at_ratio_from_start(&Euclidean, k as f64 / count as f64)
                .map(|p| p.0)
        })
        .collect();
    let n = samples.len();
    let sigma = sigma_m / (length / count as f64);
    let radius = (3.0 * sigma).ceil() as isize;
    let weights: Vec<f64> = (-radius..=radius)
        .map(|i| (-((i * i) as f64) / (2.0 * sigma * sigma)).exp())
        .collect();
    // A closed ring repeats its first point last; it wraps over the others.
    let ring = if closed { n - 1 } else { n };
    let smoothed: Vec<Coord<f64>> = (0..n)
        .map(|i| {
            if !closed && (i == 0 || i == n - 1) {
                return samples[i];
            }
            // Towards the ends of an open line the window shrinks to stay symmetric.
            let reach = if closed {
                radius
            } else {
                radius.min(i as isize).min((n - 1 - i) as isize)
            };
            let (mut x, mut y, mut w) = (0.0, 0.0, 0.0);
            for d in -reach..=reach {
                let j = (i as isize + d).rem_euclid(ring as isize) as usize;
                let weight = weights[(d + radius) as usize];
                x += samples[j].x * weight;
                y += samples[j].y * weight;
                w += weight;
            }
            Coord { x: x / w, y: y / w }
        })
        .collect();
    let mut smoothed = smoothed;
    if closed {
        smoothed[n - 1] = smoothed[0];
    }
    LineString::new(smoothed)
}

/// The median drop of `terrain` across `line`: the height on one side minus the other,
/// [`DROP_PROBE_M`] from the line, every [`DROP_STEP_M`]. `None` where the terrain says nothing.
fn drop_across(line: &LineString<f64>, terrain: &Raster<f32>) -> Option<f64> {
    let height = |e: f64, n: f64| bilinear(terrain, e, n).map(f64::from);
    let mut drops: Vec<f64> = Vec::new();
    for segment in line.lines() {
        let (dx, dy) = (segment.dx(), segment.dy());
        let length = dx.hypot(dy);
        if length < 1e-6 {
            continue;
        }
        let (ue, un) = (dx / length, dy / length);
        let count = (length / DROP_STEP_M).ceil().max(1.0) as usize;
        for k in 0..count {
            let t = (k as f64 + 0.5) / count as f64;
            let (e, n) = (segment.start.x + dx * t, segment.start.y + dy * t);
            let left = height(e - un * DROP_PROBE_M, n + ue * DROP_PROBE_M);
            let right = height(e + un * DROP_PROBE_M, n - ue * DROP_PROBE_M);
            if let (Some(left), Some(right)) = (left, right) {
                drops.push((left - right).abs());
            }
        }
    }
    if drops.is_empty() {
        return None;
    }
    drops.sort_by(f64::total_cmp);
    Some(drops[drops.len() / 2])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::points::test_support::{Fixture, fixture};
    use crate::raster::GridSpec;
    use geo::Contains as _;
    use geo_types::Point;

    type P = (f64, f64);

    /// The point `at` (metres on the map from the reference point) in the map's CRS.
    fn projected(f: &Fixture, at: P) -> P {
        let p = f.georef.map_to_projected(f.at(at));
        (p.easting, p.northing)
    }

    /// Whether `point` (map CRS) lies in one of `rock`.
    fn in_rock(rock: &[Polygon<f64>], point: P) -> bool {
        rock.iter()
            .any(|r| r.contains(&Point::new(point.0, point.1)))
    }

    fn cliff(f: &Fixture, points: &[P]) -> MapObject {
        MapObject {
            symbol_id: f.id(CLIFF_CODE),
            kind: MapObjectKind::Path,
            coords: points.iter().map(|&p| f.at(p)).collect(),
            tags: vec![("freemapper:source".to_string(), "osm".to_string())],
        }
    }

    /// A terrain 400 m square around the reference point, 1 m cells, with `z(de, dn)` at the
    /// cell centres, in metres right of and above the reference point on the map (the map is
    /// turned against the grid, so the heights follow the map's axes like the test lines).
    fn terrain(f: &Fixture, z: impl Fn(f64, f64) -> f32) -> Raster<f32> {
        let (e0, n0) = (
            f.georef.ref_projected.easting,
            f.georef.ref_projected.northing,
        );
        let grid = GridSpec {
            west: e0 - 200.0,
            north: n0 + 200.0,
            pixel_size: 1.0,
            width: 400,
            height: 400,
        };
        let mut r = Raster::filled(grid, 0.0);
        for y in 0..400 {
            for x in 0..400 {
                let at = f
                    .georef
                    .projected_to_map(ProjectedPoint {
                        easting: e0 - 199.5 + f64::from(x),
                        northing: n0 + 199.5 - f64::from(y),
                    })
                    .unwrap();
                r.set(
                    x,
                    y,
                    z(f64::from(at.x) / f.metre(), -f64::from(at.y) / f.metre()),
                );
            }
        }
        r
    }

    /// Steps of `heights[i]` metres down, southwards, at the norths `steps[i]`.
    fn terraces(steps: &'static [(f64, f32)]) -> impl Fn(f64, f64) -> f32 {
        move |_, dn| {
            100.0
                + steps
                    .iter()
                    .filter(|(n, _)| dn > *n)
                    .map(|(_, h)| h)
                    .sum::<f32>()
        }
    }

    fn run(
        f: &Fixture,
        map: &mut Vec<MapObject>,
        terrain: Option<&Raster<f32>>,
    ) -> (Vec<MapObject>, Vec<Polygon<f64>>) {
        let mut candidates = Vec::new();
        let rock = generalize_cliffs(map, &mut candidates, terrain, f.kind, &f.georef, &f.set);
        (candidates, rock)
    }

    fn generalized(o: &MapObject) -> Option<&str> {
        o.tags
            .iter()
            .find(|(k, _)| k == GENERALIZED_KEY)
            .map(|(_, v)| v.as_str())
    }

    fn hidden(o: &MapObject) -> bool {
        o.tags
            .iter()
            .any(|(k, v)| k == HIDDEN_KEY && v == GENERALIZED)
    }

    #[test]
    fn a_low_cliff_goes_to_the_candidates_hidden() {
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, terraces(&[(0.0, 1.5)]));
        let mut map = vec![cliff(&f, &[(-50.0, 0.0), (50.0, 0.0)])];
        let (candidates, rock) = run(&f, &mut map, Some(&ground));
        assert!(map.is_empty());
        assert_eq!(candidates.len(), 1);
        assert!(hidden(&candidates[0]));
        assert_eq!(generalized(&candidates[0]), Some("cliff lower than 3 m"));
        assert!(rock.is_empty(), "a low cliff stops no contour");
    }

    #[test]
    fn of_two_cliffs_too_close_the_higher_stays() {
        // 1:15 000: 0.25 + 0.175 + 0.575 mm = 15 m between the lines.
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, terraces(&[(0.0, 4.0), (10.0, 12.0)]));
        let mut map = vec![
            cliff(&f, &[(-60.0, 0.0), (60.0, 0.0)]),
            cliff(&f, &[(-60.0, 10.0), (60.0, 10.0)]),
        ];
        let (candidates, rock) = run(&f, &mut map, Some(&ground));
        assert_eq!(map.len(), 1);
        assert_eq!(
            map[0].coords[0],
            f.at((-60.0, 10.0)),
            "the 12 m cliff stays"
        );
        assert_eq!(candidates.len(), 1);
        assert!(hidden(&candidates[0]));
        assert_eq!(
            generalized(&candidates[0]),
            Some("cliff closer than 0.25 mm to a higher one")
        );
        // Both are tall: the contours stop at both.
        assert!(in_rock(&rock, projected(&f, (0.0, 0.0))));
        assert!(in_rock(&rock, projected(&f, (0.0, 10.0))));
    }

    #[test]
    fn cliffs_far_enough_apart_both_stay() {
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, terraces(&[(0.0, 4.0), (20.0, 12.0)]));
        let mut map = vec![
            cliff(&f, &[(-60.0, 0.0), (60.0, 0.0)]),
            cliff(&f, &[(-60.0, 20.0), (60.0, 20.0)]),
        ];
        let (candidates, _) = run(&f, &mut map, Some(&ground));
        assert_eq!(map.len(), 2);
        assert!(candidates.is_empty());
    }

    #[test]
    fn a_cliff_shorter_than_the_norm_goes() {
        // 0.6 mm at 1:15 000: 9 m.
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, terraces(&[(0.0, 6.0)]));
        let mut map = vec![cliff(&f, &[(0.0, 0.0), (7.0, 0.0)])];
        let (candidates, _) = run(&f, &mut map, Some(&ground));
        assert!(map.is_empty());
        assert_eq!(
            generalized(&candidates[0]),
            Some("cliff shorter than 0.6 mm")
        );
    }

    #[test]
    fn a_closed_cliff_round_a_pillar_becomes_206() {
        let f = fixture(MapKind::Forest);
        // A tower 20 m square and 15 m high.
        let ground = terrain(&f, |de, dn| {
            if de.abs() < 10.0 && dn.abs() < 10.0 {
                115.0
            } else {
                100.0
            }
        });
        let ring = [
            (-10.0, -10.0),
            (10.0, -10.0),
            (10.0, 10.0),
            (-10.0, 10.0),
            (-10.0, -10.0),
        ];
        let mut map = vec![cliff(&f, &ring)];
        let (candidates, rock) = run(&f, &mut map, Some(&ground));
        assert!(candidates.is_empty(), "the pillar is the same rock");
        assert_eq!(map.len(), 1);
        assert_eq!(map[0].symbol_id, f.id(PILLAR_CODE));
        assert_eq!(
            generalized(&map[0]),
            Some("rock pillar from a closed cliff")
        );
        assert!(in_rock(&rock, projected(&f, (0.0, 0.0))));
    }

    #[test]
    fn a_cliff_beside_a_pillar_goes() {
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, |de, dn| {
            if de.abs() < 10.0 && dn.abs() < 10.0 {
                120.0
            } else if dn > 15.0 {
                110.0
            } else {
                100.0
            }
        });
        let ring = [
            (-10.0, -10.0),
            (10.0, -10.0),
            (10.0, 10.0),
            (-10.0, 10.0),
            (-10.0, -10.0),
        ];
        let mut map = vec![cliff(&f, &ring), cliff(&f, &[(-40.0, 15.0), (40.0, 15.0)])];
        let (candidates, _) = run(&f, &mut map, Some(&ground));
        assert_eq!(map.len(), 1);
        assert_eq!(map[0].symbol_id, f.id(PILLAR_CODE));
        assert_eq!(
            generalized(&candidates[0]),
            Some("cliff beside a rock pillar")
        );
    }

    #[test]
    fn without_terrain_every_cliff_counts_as_tall() {
        let f = fixture(MapKind::Forest);
        let mut map = vec![cliff(&f, &[(-50.0, 0.0), (50.0, 0.0)])];
        let (candidates, rock) = run(&f, &mut map, None);
        assert_eq!(map.len(), 1);
        assert!(candidates.is_empty());
        assert!(!rock.is_empty());
    }

    #[test]
    fn smoothing_takes_out_the_zigzag_and_keeps_the_ends() {
        // A face 100 m long with niches 2 m deep every 4 m, as OSM traces it.
        let zigzag: LineString<f64> = (0..=50)
            .map(|i| Coord {
                x: f64::from(i) * 2.0,
                y: if i % 2 == 0 { 0.0 } else { 2.0 },
            })
            .collect();
        // 0.3 mm at 1:10 000 with symbols at 150 %: 4.5 m.
        let smoothed = smooth(&zigzag, 4.5, false);
        assert_eq!(smoothed.0.first(), zigzag.0.first());
        assert_eq!(smoothed.0.last(), zigzag.0.last());
        let inner = &smoothed.0[5..smoothed.0.len() - 5];
        assert!(inner.iter().all(|c| (c.y - 1.0).abs() < 0.1), "{inner:?}");
        // A bend of the face as long as the face stays.
        let bend: LineString<f64> = vec![(0.0, 0.0), (50.0, 0.0), (50.0, 50.0)].into();
        let smoothed = smooth(&bend, 4.5, false);
        let corner = smoothed
            .0
            .iter()
            .map(|c| (c.x - 50.0).hypot(c.y))
            .fold(f64::INFINITY, f64::min);
        assert!(corner < 3.0, "the corner is rounded, not cut off: {corner}");
    }

    #[test]
    fn a_sprint_map_keeps_its_cliffs() {
        let f = fixture(MapKind::Sprint);
        let ground = terrain(&f, terraces(&[(0.0, 1.0)]));
        let mut map = vec![
            cliff(&f, &[(-50.0, 0.0), (50.0, 0.0)]),
            cliff(&f, &[(-50.0, 2.0), (50.0, 2.0)]),
        ];
        let (candidates, rock) = run(&f, &mut map, Some(&ground));
        assert_eq!(map.len(), 2);
        assert!(candidates.is_empty() && rock.is_empty());
    }
}
