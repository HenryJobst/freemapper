//! The notches of a surveyed line: small detours that leave a line and come back to it, such as
//! a kerb run around a tree pit (ADR 0149). A set names the layers whose lines lose them
//! (`SetDrawing::notch_free_layers`).

use crate::omap::MapObject;
use crate::omap::coord::FLAG_CURVE_START;

/// A kerb detour that encloses less than this (1.5 mm² on paper, in (1/1000 mm)²) is no feature
/// of the street. ISSprOM 2019-2 §3.3 asks 1.0 mm² for the smallest area of a brown dot screen,
/// which paving is; the surveyed tree pits with their lead-in measure up to 1.1 mm² at 1:4000, so
/// the limit takes half a square millimetre on top.
const KERB_NOTCH_MAX_AREA: f64 = 1.5e6;
/// A detour longer than this on paper (10 mm, 40 m at 1:4000) is never a pit.
const KERB_NOTCH_MAX_PATH: f64 = 10_000.0;
/// A detour wider than this on paper (2.5 mm, 10 m at 1:4000) is no single pit.
const KERB_NOTCH_MAX_CHORD: f64 = 2_500.0;
/// A detour is a pit when it reaches out at least this share of its chord: a quarter-circle
/// corner reaches 0.21, a pit with its lead-in 0.4 and more.
const KERB_NOTCH_MIN_DEPTH: f64 = 0.3;
/// The kerb before and after a detour has to run along its chord: the cosine of the angle
/// between them is at least this (20 degrees). Without it a chord to the apex of a slot, or across
/// a step of the kerb, would count as a notch and draw a spike or a diagonal.
const KERB_NOTCH_MIN_COS: f64 = 0.94;
/// How far from the detour's ends the kerb direction is taken (0.5 mm), to skip survey jitter.
const KERB_NOTCH_REFERENCE: f64 = 500.0;
/// A line of a layer a set names in `notch_free_layers` without the notches cut into it (ADR
/// 0149): Berlin's street survey runs the kerb (501.1) around each tree pit in a parking strip,
/// and the pit then shows as a hook along the street edge. Each detour that returns to the line within [`KERB_NOTCH_MAX_PATH`] of way and
/// [`KERB_NOTCH_MAX_CHORD`] of chord, reaches out [`KERB_NOTCH_MIN_DEPTH`] of its chord and
/// encloses less than [`KERB_NOTCH_MAX_AREA`] is replaced by its chord. A closed kerb ring below that area, the
/// kerb around a lone pit, goes. `None` when the object goes; a line with curve points stays as it is. The close flag of a ring does not count as one.
pub(crate) fn drop_notches(mut object: MapObject) -> Option<MapObject> {
    if object
        .coords
        .iter()
        .any(|c| c.flags & FLAG_CURVE_START != 0)
    {
        return Some(object);
    }
    let points: Vec<(f64, f64)> = object
        .coords
        .iter()
        .map(|c| (f64::from(c.x), f64::from(c.y)))
        .collect();
    let n = points.len();
    if n < 3 {
        return Some(object);
    }
    // Twice the signed area of the loop `points[i..=j]` closed by its chord.
    let cross = |a: (f64, f64), b: (f64, f64)| a.0 * b.1 - b.0 * a.1;
    if points[0] == points[n - 1] {
        let area2: f64 = points.windows(2).map(|w| cross(w[0], w[1])).sum();
        if area2.abs() / 2.0 < KERB_NOTCH_MAX_AREA {
            return None;
        }
    }
    let mut kept = vec![object.coords[0]];
    let mut i = 0;
    while i < n - 1 {
        let mut best = None;
        let (mut path, mut area2) = (0.0, 0.0);
        for j in i + 1..n {
            path += (points[j].0 - points[j - 1].0).hypot(points[j].1 - points[j - 1].1);
            if path > KERB_NOTCH_MAX_PATH {
                break;
            }
            area2 += cross(points[j - 1], points[j]);
            let chord = (points[j].0 - points[i].0).hypot(points[j].1 - points[i].1);
            let closed = area2 + cross(points[j], points[i]);
            if j < i + 2
                || chord > KERB_NOTCH_MAX_CHORD
                || closed.abs() / 2.0 >= KERB_NOTCH_MAX_AREA
            {
                continue;
            }
            // The farthest the detour gets from the chord.
            let depth = points[i + 1..j]
                .iter()
                .map(|p| {
                    ((p.0 - points[i].0) * (points[j].1 - points[i].1)
                        - (p.1 - points[i].1) * (points[j].0 - points[i].0))
                        .abs()
                        / chord.max(1.0)
                })
                .fold(0.0, f64::max);
            if depth >= KERB_NOTCH_MIN_DEPTH * chord && runs_along(&points, i, j, chord) {
                best = Some(j);
            }
        }
        let next = best.unwrap_or(i + 1);
        kept.push(object.coords[next]);
        i = next;
    }
    object.coords = kept;
    Some(object)
}

/// Whether the kerb before `points[i]` and after `points[j]` runs along the chord `i`..`j`
/// ([`KERB_NOTCH_MIN_COS`]). A side that has no point [`KERB_NOTCH_REFERENCE`] away does not
/// count; a chord of no length always passes.
fn runs_along(points: &[(f64, f64)], i: usize, j: usize, chord: f64) -> bool {
    if chord < 1.0 {
        return true;
    }
    let along = |from: usize, forward: bool| -> Option<(f64, f64)> {
        let origin = points[from];
        let mut k = from;
        loop {
            k = if forward { k + 1 } else { k.checked_sub(1)? };
            let p = *points.get(k)?;
            let (dx, dy) = (p.0 - origin.0, p.1 - origin.1);
            if dx.hypot(dy) >= KERB_NOTCH_REFERENCE {
                return Some(if forward { (dx, dy) } else { (-dx, -dy) });
            }
        }
    };
    let chord_dir = (points[j].0 - points[i].0, points[j].1 - points[i].1);
    [along(i, false), along(j, true)]
        .into_iter()
        .flatten()
        .all(|(dx, dy)| {
            (dx * chord_dir.0 + dy * chord_dir.1) / (dx.hypot(dy) * chord) >= KERB_NOTCH_MIN_COS
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::omap::{MapCoord, MapObjectKind};
    use crate::project::MapKind;

    /// A kerb line of the survey from `(x, y)` points in 1/1000 mm on paper (1:4000: 250 per
    /// ground metre).
    fn kerb(points: &[(i32, i32)]) -> MapObject {
        let set = MapKind::Sprint.symbol_set(4000);
        MapObject {
            symbol_id: set.id_for_code("501.1").unwrap(),
            kind: MapObjectKind::Path,
            coords: points.iter().map(|&(x, y)| MapCoord::new(x, y)).collect(),
            tags: vec![(
                "freemapper:source".into(),
                "geoportal-berlin/sb-bordstein".into(),
            )],
        }
    }

    fn xy(object: &MapObject) -> Vec<(i32, i32)> {
        object.coords.iter().map(|c| (c.x, c.y)).collect()
    }

    /// A tree pit cut into the parking strip: the kerb leaves the straight line, runs around a
    /// 2 m by 4.5 m slot and comes back (about 0.8 mm² at 1:4000, the survey's own shape).
    fn kerb_with_pit() -> MapObject {
        kerb(&[
            (0, 0),
            (10_000, 0),
            (10_200, 400),
            (10_200, 1_100),
            (10_450, 1_250),
            (10_700, 1_100),
            (10_700, 400),
            (10_900, 0),
            (20_000, 0),
        ])
    }

    #[test]
    fn tree_pit_notch_leaves_the_kerb_straight() {
        let kept = drop_notches(kerb_with_pit()).expect("a long kerb stays");
        assert_eq!(xy(&kept), [(0, 0), (10_000, 0), (10_900, 0), (20_000, 0)]);
        assert_eq!(kept.symbol_id, kerb_with_pit().symbol_id);
        assert_eq!(kept.tags, kerb_with_pit().tags);
    }

    #[test]
    fn wide_trapezoid_bump_at_a_tree_is_straightened() {
        // The surveyed kerb at a Viktoriapark street tree (Li_13.04.2015_719): it leaves the edge
        // on a slant, rounds the tree and comes back on a slant. Chord 1.7 mm, way 3.2 mm, 1.3 mm²
        // at 1:4000: a way of only 1.8 times the chord, so the first rule let it stand.
        let bump = kerb(&[
            (49_236, -57_276),
            (50_228, -57_491),
            (50_434, -58_123),
            (50_639, -58_755),
            (50_694, -58_848),
            (50_785, -58_910),
            (50_890, -58_927),
            (50_996, -58_898),
            (51_078, -58_828),
            (51_539, -58_308),
            (51_934, -57_862),
            (52_257, -57_932),
            (52_818, -58_054),
            (53_241, -58_146),
            (54_280, -58_348),
        ]);
        let kept = drop_notches(bump).unwrap();
        assert!(
            kept.coords.iter().all(|c| c.y > -58_500),
            "the bump is gone: {:?}",
            xy(&kept)
        );
        assert_eq!(xy(&kept)[..2], [(49_236, -57_276), (50_228, -57_491)]);
        assert_eq!(xy(&kept).last(), Some(&(54_280, -58_348)));
    }

    #[test]
    fn a_notch_ends_on_the_line_not_at_its_apex() {
        // Viktoriapark, street edge along the park (Li_15.04.2015_251): a slanted trapezoid slot
        // at a tree. The rule of ADR 0071's first amendment cut from the vertex before it to the
        // slot's apex and drew a spike; the slot has to go back to the line at its far end.
        let slot = kerb(&[
            (-25_211, -40_924),
            (-23_160, -41_374),
            (-22_811, -41_451),
            (-22_463, -41_527),
            (-21_480, -41_743),
            (-19_813, -42_109),
            (-19_390, -42_507),
            (-19_861, -43_003),
            (-19_919, -43_093),
            (-19_935, -43_208),
            (-19_901, -43_317),
            (-19_832, -43_396),
            (-19_733, -43_442),
            (-19_616, -43_443),
            (-19_514, -43_395),
            (-19_476, -43_359),
            (-18_873, -42_706),
            (-18_759, -42_582),
            (-18_573, -42_382),
            (-14_659, -43_242),
        ]);
        let kept = xy(&drop_notches(slot).unwrap());
        assert!(
            kept.iter().all(|&(x, y)| x > -19_000 || y > -42_200),
            "no spike left of the exit: {kept:?}"
        );
        assert!(kept.contains(&(-18_573, -42_382)), "{kept:?}");
    }

    #[test]
    fn a_step_of_the_kerb_is_not_cut_diagonally() {
        // Viktoriapark, left of the X mark: the kerb steps back by 5.7 m and takes a tab on the
        // way. There is no line to return to, so the corner stays as the survey has it.
        let step = kerb(&[
            (-40_386, -37_712),
            (-35_991, -38_676),
            (-35_463, -38_980),
            (-35_946, -39_587),
            (-35_962, -39_671),
            (-35_947, -39_764),
            (-35_898, -39_848),
            (-35_825, -39_907),
            (-35_744, -39_935),
            (-35_391, -40_003),
            (-34_949, -40_102),
            (-34_078, -40_298),
            (-33_608, -40_399),
            (-32_016, -40_745),
            (-31_517, -40_854),
        ]);
        assert_eq!(xy(&drop_notches(step.clone()).unwrap()), xy(&step));
    }

    #[test]
    fn a_closed_ring_with_the_close_flag_is_dropped_too() {
        // The survey ring as the builder writes it: the last coordinate carries FLAG_CLOSE and
        // FLAG_HOLE (flags 18). A pit ring of 1.8 mm by 0.8 mm (1.2 mm²) from Viktoriapark.
        use crate::omap::coord::{FLAG_CLOSE, FLAG_HOLE};
        let mut ring = kerb(&[(0, 0), (1_800, 0), (1_800, 800), (0, 800), (0, 0)]);
        ring.coords[4].flags = FLAG_CLOSE | FLAG_HOLE;
        assert!(drop_notches(ring).is_none());
    }

    #[test]
    fn a_bay_larger_than_the_minimum_area_stays() {
        // A 12 m by 6 m bay (4.5 mm²) is a feature of the street, not a pit.
        let bay = kerb(&[
            (0, 0),
            (10_000, 0),
            (10_000, 1_500),
            (13_000, 1_500),
            (13_000, 0),
            (20_000, 0),
        ]);
        assert_eq!(xy(&drop_notches(bay.clone()).unwrap()), xy(&bay));
    }

    #[test]
    fn a_curve_is_not_a_notch() {
        // A quarter circle of 3 mm radius at the corner: path and chord are nearly equal.
        let corner: Vec<(i32, i32)> = (0..=12)
            .map(|i| {
                let a = f64::from(i) / 12.0 * std::f64::consts::FRAC_PI_2;
                (
                    (3_000.0 * a.sin()) as i32,
                    (3_000.0 * (1.0 - a.cos())) as i32,
                )
            })
            .collect();
        let line = kerb(&corner);
        assert_eq!(xy(&drop_notches(line.clone()).unwrap()), xy(&line));
    }

    #[test]
    fn a_tiny_closed_kerb_ring_is_dropped() {
        // A tree pit ringed by its own kerb: 2 m by 2.5 m, 0.3 mm².
        let ring = kerb(&[(0, 0), (500, 0), (500, 625), (0, 625), (0, 0)]);
        assert!(drop_notches(ring).is_none());
        // A square of 12 m by 12 m stays.
        let square = kerb(&[(0, 0), (3_000, 0), (3_000, 3_000), (0, 3_000), (0, 0)]);
        assert!(drop_notches(square).is_some());
    }
}
