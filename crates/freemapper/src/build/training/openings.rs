//! Step 1b of the training level: an opening between the ends of walls and fences narrower than
//! the norm's is widened to it.
//!
//! ISSprOM 2019-2 §3.3 gives the opening of walls and fences as 1.0 mm (ADR 0068). The data hold
//! narrower ones, a gate of 1.5 m in a fence round a pitch, which at 1:4000 is 0.4 mm: on the
//! training map it reads as a fence drawn wrong. Of two ends of impassable walls and fences (the
//! two ends of one line among them) of one source that come closer than the opening but lie at
//! least [`MIN_OPENING_M`] apart, both are shortened along their lines by the same length until they lie the
//! opening apart (round a corner that is more than half of what was missing); the nearest pairs
//! go first and an end is used once. Ends of two sources (OSM and the cadastre drawing one fence a
//! little apart) and narrower gaps are the data's, not a way through, and stay. The base map
//! keeps the measured positions.

use crate::build::barriers::PIECE_MIN_MM;
use freemapper_contract::tags::SOURCE_KEY;

use crate::georef::Georeferencing;
use crate::measure::{SymbolScale, map_units_per_metre};
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet};

use super::{add_generalized, is_closed};

/// Impassable wall and impassable fence: a runner passes them only where they open.
const CODES: [&str; 2] = ["515", "518"];
/// The `freemapper:generalized` value of a line whose end gave way.
const WIDENED: &str = "opening widened to the norm (ISSprOM 2019-2 §3.3)";

/// A gap narrower than this on the ground (metres) is no way through for a runner.
const MIN_OPENING_M: f64 = 0.5;

/// An opening this much (map units) under the norm counts as wide enough: the widened ends are
/// rounded to whole map units.
const TOLERANCE: f64 = 2.0;

type P = (f64, f64);

/// Widens the openings between the ends of walls and fences (see the module). Returns how many
/// objects changed; a line shortened to nothing goes.
pub(super) fn widen_openings(
    objects: &mut Vec<MapObject>,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> usize {
    let ids: Vec<u32> = CODES
        .iter()
        .filter_map(|c| symbol_set.id_for_code(c))
        .collect();
    let scale = SymbolScale::drawn_with(symbol_set, georef.scale_denominator);
    let opening = PIECE_MIN_MM.on_paper(scale).0 * 1000.0;
    let narrowest = MIN_OPENING_M * map_units_per_metre(georef.scale_denominator);
    let source = |i: usize| {
        objects[i]
            .tags
            .iter()
            .find(|(k, _)| k == SOURCE_KEY)
            .map(|(_, v)| v.as_str())
    };
    // The ends: object, whether it is the first point, position (map units).
    let ends: Vec<(usize, bool, P)> = objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            ids.contains(&o.symbol_id)
                && matches!(o.kind, MapObjectKind::Path)
                && o.coords.len() >= 2
                && o.coords.iter().all(|c| c.flags == 0)
                && !is_closed(o)
        })
        .flat_map(|(i, o)| {
            [
                (i, true, xy(&o.coords[0])),
                (i, false, xy(&o.coords[o.coords.len() - 1])),
            ]
        })
        .collect();
    let mut pairs: Vec<(f64, usize, usize)> = Vec::new();
    for a in 0..ends.len() {
        for b in a + 1..ends.len() {
            let d = distance(ends[a].2, ends[b].2);
            if d >= narrowest && d < opening - TOLERANCE && source(ends[a].0) == source(ends[b].0) {
                pairs.push((d, a, b));
            }
        }
    }
    pairs.sort_by(|x, y| x.0.total_cmp(&y.0));
    let mut used = vec![false; ends.len()];
    // How much each end gives way: object → (from its start, from its end).
    let mut trims: std::collections::BTreeMap<usize, (f64, f64)> = Default::default();
    for (_, a, b) in pairs {
        if used[a] || used[b] {
            continue;
        }
        used[a] = true;
        used[b] = true;
        let line = |(object, first, _): (usize, bool, P)| {
            let mut points: Vec<P> = objects[object].coords.iter().map(xy).collect();
            if !first {
                points.reverse();
            }
            points
        };
        let (line_a, line_b) = (line(ends[a]), line(ends[b]));
        let cut = equal_cut(&line_a, &line_b, opening);
        if cut <= 0.0 {
            continue;
        }
        for &(object, first, _) in [&ends[a], &ends[b]] {
            let trim = trims.entry(object).or_default();
            match first {
                true => trim.0 += cut,
                false => trim.1 += cut,
            }
        }
    }
    let changed = trims.len();
    let mut gone = Vec::new();
    for (&i, &(start, end)) in &trims {
        let object = &mut objects[i];
        let mut points: Vec<P> = object.coords.iter().map(xy).collect();
        let total = length(&points);
        if start + end >= total {
            gone.push(i);
            continue;
        }
        points = cut_start(&points, start);
        points.reverse();
        points = cut_start(&points, end);
        points.reverse();
        object.coords = points
            .into_iter()
            .map(|(x, y)| MapCoord::new(x.round() as i32, y.round() as i32))
            .collect();
        add_generalized(object, WIDENED);
    }
    for i in gone.into_iter().rev() {
        objects.remove(i);
    }
    changed
}

fn xy(c: &MapCoord) -> P {
    (f64::from(c.x), f64::from(c.y))
}

fn distance(a: P, b: P) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

fn length(points: &[P]) -> f64 {
    points.windows(2).map(|w| distance(w[0], w[1])).sum()
}

/// The length both lines (each starting at the end that opens) give way by so that their new
/// starts lie `opening` apart: the first step of a quarter opening that reaches it, then halved.
/// The search stops at half of the shorter line (the two ends of one line meet again past its
/// middle); 0 when no length reaches the opening.
fn equal_cut(a: &[P], b: &[P], opening: f64) -> f64 {
    let gap = |cut: f64| distance(cut_start(a, cut)[0], cut_start(b, cut)[0]);
    let limit = length(a).min(length(b)) / 2.0;
    let step = opening / 4.0;
    let mut hi = step;
    while gap(hi.min(limit)) < opening {
        if hi >= limit {
            return 0.0;
        }
        hi += step;
    }
    let (mut lo, mut hi) = ((hi - step).max(0.0), hi.min(limit));
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        match gap(mid) < opening {
            true => lo = mid,
            false => hi = mid,
        }
    }
    hi
}

/// `points` without its first `cut` (map units) along the line.
fn cut_start(points: &[P], cut: f64) -> Vec<P> {
    let mut left = cut;
    for (k, w) in points.windows(2).enumerate() {
        let step = distance(w[0], w[1]);
        if step > left {
            let t = left / step;
            let at = (
                w[0].0 + t * (w[1].0 - w[0].0),
                w[0].1 + t * (w[1].1 - w[0].1),
            );
            return std::iter::once(at)
                .chain(points[k + 1..].iter().copied())
                .collect();
        }
        left -= step;
    }
    points[points.len() - 1..].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::barriers::PIECE_MIN_MM;
    use crate::build::training::tests::{SCALE, georef_at, isom, line};
    use crate::measure::SymbolScale;

    /// The norm's opening on paper at the scale of the tests (mm).
    fn opening(set: &SymbolSet) -> f64 {
        PIECE_MIN_MM.on_paper(SymbolScale::drawn_with(set, SCALE)).0
    }

    fn widen(set: &SymbolSet, objects: &mut Vec<MapObject>) -> usize {
        widen_openings(objects, &georef_at(SCALE), set)
    }

    /// The first and last point of a line (paper mm).
    fn ends(object: &MapObject) -> ((f64, f64), (f64, f64)) {
        let mm = |c: &crate::omap::MapCoord| (f64::from(c.x) / 1000.0, f64::from(c.y) / 1000.0);
        (mm(&object.coords[0]), mm(object.coords.last().unwrap()))
    }

    fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
        (a.0 - b.0).hypot(a.1 - b.1)
    }

    #[test]
    fn narrow_opening_of_a_fence_round_a_square_is_widened() {
        let set = isom();
        // A square of 10 mm whose ends stop 0.4 mm apart on its first side.
        let mut objects = vec![line(
            &set,
            "518",
            &[
                (0.4, 0.0),
                (10.0, 0.0),
                (10.0, 10.0),
                (0.0, 10.0),
                (0.0, 0.0),
            ],
        )];
        assert_eq!(widen(&set, &mut objects), 1);
        let (first, last) = ends(&objects[0]);
        assert!(
            (distance(first, last) - opening(&set)).abs() < 0.005,
            "{first:?} {last:?}"
        );
        // Both ends gave way by the same length round the corner: the first along the top side,
        // the last back up the left side.
        assert!(
            first.1.abs() < 0.005 && last.0.abs() < 0.005,
            "{first:?} {last:?}"
        );
        assert!(
            ((first.0 - 0.4) - last.1).abs() < 0.005,
            "{first:?} {last:?}"
        );
        assert!(
            objects[0]
                .tags
                .iter()
                .any(|(k, v)| k == "freemapper:generalized" && v.contains("opening"))
        );
        // Wide enough now: nothing more to do.
        assert_eq!(widen(&set, &mut objects), 0);
    }

    #[test]
    fn narrow_opening_between_a_wall_and_a_fence_is_widened() {
        let set = isom();
        let mut objects = vec![
            line(&set, "515", &[(0.0, 0.0), (10.0, 0.0)]),
            line(&set, "518", &[(10.5, 0.0), (20.0, 0.0)]),
        ];
        assert_eq!(widen(&set, &mut objects), 2);
        let gap = distance(ends(&objects[0]).1, ends(&objects[1]).0);
        assert!((gap - opening(&set)).abs() < 0.005, "{gap}");
    }

    fn sourced(mut object: MapObject, source: &str) -> MapObject {
        object
            .tags
            .push((SOURCE_KEY.to_string(), source.to_string()));
        object
    }

    #[test]
    fn touching_narrow_wide_and_mixed_openings_stay() {
        let set = isom();
        let wide = 2.0 * opening(&set);
        let mut objects = vec![
            // Ends that touch are one line to the runner.
            line(&set, "518", &[(0.0, 0.0), (10.0, 0.0)]),
            line(&set, "518", &[(10.0, 0.0), (20.0, 5.0)]),
            // Wide enough already.
            line(&set, "515", &[(0.0, 20.0), (10.0, 20.0)]),
            line(&set, "515", &[(10.0 + wide, 20.0), (30.0, 20.0)]),
            // Lines of two sources drawing one fence a little apart.
            sourced(line(&set, "518", &[(0.0, 60.0), (10.0, 60.0)]), "osm"),
            sourced(line(&set, "518", &[(10.4, 60.0), (20.0, 60.0)]), "alkis"),
            // A gap of 0.3 m (0.03 mm at 1:10 000) is no way through.
            line(&set, "518", &[(0.0, 80.0), (10.0, 80.0)]),
            line(&set, "518", &[(10.03, 80.0), (20.0, 80.0)]),
            // A fence a runner may cross needs no opening.
            line(&set, "516", &[(0.0, 40.0), (10.0, 40.0)]),
            line(&set, "516", &[(10.3, 40.0), (20.0, 40.0)]),
        ];
        let before = objects.clone();
        assert_eq!(widen(&set, &mut objects), 0);
        assert_eq!(objects, before);
    }
}
