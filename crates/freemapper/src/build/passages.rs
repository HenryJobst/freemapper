//! Reachable building passages (ADR 0076). A 522 candidate that is a passage through or under a
//! building matters only if a runner can get to it: from walkable ground at one end, or through
//! a chain of passages (the Lübecker Höfe und Gänge). A passage that touches only buildings, an
//! area that shall not be entered, water, or a courtyard that has no way out is tagged
//! [`HIDDEN_KEY`]` = unreachable`. It stays among the candidates and is left out of the training
//! map, as the points under a 520 area are.

use geo::{
    Area as _, BooleanOps as _, BoundingRect as _, Buffer as _, Intersects as _, unary_union,
};
use geo_types::{MultiPolygon, Polygon, Rect};

use crate::build::geometry::object_polygon;
use crate::build::points::{HIDDEN_KEY, OUT_OF_BOUNDS_CODE, projected};
use crate::crs::BoundingBox;
use crate::georef::Georeferencing;
use crate::measure::PaperMm;
use crate::omap::{MapObject, SymbolSet};

/// The value of [`HIDDEN_KEY`] on a passage no runner can reach.
pub const UNREACHABLE: &str = "unreachable";
/// ISSprOM 522 canopy, the symbol of the passages.
const PASSAGE_CODE: &str = "522";
/// A passage touches what lies within this distance on paper (mm) of it: 2 m at 1:4000. It
/// bridges the gap between a passage that ends a little inside its building (an OSM way ends
/// at the doorway, the cadastre part at the wall) and the street.
pub const REACH_TOLERANCE_MM: PaperMm = PaperMm(0.5);

/// Tags the 522 passages among `objects` that cannot be reached ([`UNREACHABLE`]).
///
/// Walkable ground is the clip window less the `footprints` (521 buildings), the 520 areas
/// (among `objects` and `out_of_bounds`) and the water. A connected piece of it that touches the
/// edge of the window belongs to the street network; the others are enclosed (courtyards). A
/// passage is reachable when, together with the passages and ground pieces within
/// [`REACH_TOLERANCE_MM`] of it, it is linked to the street network.
pub fn hide_unreachable(
    objects: &mut [MapObject],
    out_of_bounds: &[MapObject],
    footprints: &[Polygon<f64>],
    water_union: &MultiPolygon<f64>,
    clip: &BoundingBox,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) {
    let Some(passage_id) = symbol_set.id_for_code(PASSAGE_CODE) else {
        return;
    };
    let polygon_of = |o: &MapObject| object_polygon(o).map(|p| projected(&p, georef));
    let passages: Vec<(usize, Polygon<f64>)> = objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.symbol_id == passage_id)
        .filter_map(|(i, o)| Some((i, polygon_of(o)?)))
        .collect();
    if passages.is_empty() {
        return;
    }

    // The walkable ground: the window less everything a runner cannot cross.
    let fence_id = symbol_set.id_for_code(OUT_OF_BOUNDS_CODE);
    let mut blocked: Vec<Polygon<f64>> = footprints.to_vec();
    blocked.extend(
        objects
            .iter()
            .chain(out_of_bounds)
            .filter(|o| Some(o.symbol_id) == fence_id)
            .filter_map(polygon_of),
    );
    blocked.extend(water_union.0.iter().cloned());
    let window = Rect::new(
        geo_types::coord! { x: clip.min_e, y: clip.min_n },
        geo_types::coord! { x: clip.max_e, y: clip.max_n },
    )
    .to_polygon();
    let ground = MultiPolygon(vec![window.clone()]).difference(&unary_union(blocked.iter()));
    let regions: Vec<Polygon<f64>> = ground
        .0
        .into_iter()
        .filter(|p| p.unsigned_area() > 0.0)
        .collect();
    let edge = window.exterior().clone();
    let street: Vec<bool> = regions.iter().map(|r| r.intersects(&edge)).collect();

    // A graph of the passages and the ground pieces, joined where they come within the tolerance.
    let tolerance = REACH_TOLERANCE_MM.metres(georef.scale_denominator);
    let reach: Vec<MultiPolygon<f64>> = passages.iter().map(|(_, p)| p.buffer(tolerance)).collect();
    let bounds = |m: &MultiPolygon<f64>| m.bounding_rect();
    let region_bounds: Vec<Option<Rect<f64>>> = regions.iter().map(|r| r.bounding_rect()).collect();
    let passage_bounds: Vec<Option<Rect<f64>>> = reach.iter().map(bounds).collect();
    let mut parent: Vec<usize> = (0..passages.len() + regions.len()).collect();
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    let join = |parent: &mut Vec<usize>, a: usize, b: usize| {
        let (a, b) = (root(parent, a), root(parent, b));
        parent[a] = b;
    };
    let overlap = |a: &Option<Rect<f64>>, b: &Option<Rect<f64>>| match (a, b) {
        (Some(a), Some(b)) => a.intersects(b),
        _ => false,
    };
    for (i, near) in reach.iter().enumerate() {
        for (j, (_, other)) in passages.iter().enumerate().skip(i + 1) {
            if overlap(&passage_bounds[i], &other.bounding_rect()) && near.intersects(other) {
                join(&mut parent, i, j);
            }
        }
        for (k, region) in regions.iter().enumerate() {
            if overlap(&passage_bounds[i], &region_bounds[k]) && near.intersects(region) {
                join(&mut parent, i, passages.len() + k);
            }
        }
    }
    let mut open = vec![false; parent.len()];
    for (k, &is_street) in street.iter().enumerate() {
        if is_street {
            let r = root(&mut parent, passages.len() + k);
            open[r] = true;
        }
    }
    for (n, &(index, _)) in passages.iter().enumerate() {
        let r = root(&mut parent, n);
        let object = &mut objects[index];
        if !open[r] && !object.tags.iter().any(|(k, _)| k == HIDDEN_KEY) {
            object
                .tags
                .push((HIDDEN_KEY.to_string(), UNREACHABLE.to_string()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::points::test_support::{Fixture, fixture};
    use crate::project::MapKind;

    fn hidden(objects: &[MapObject]) -> Vec<&str> {
        objects
            .iter()
            .filter(|o| {
                o.tags
                    .iter()
                    .any(|(k, v)| k == HIDDEN_KEY && v == UNREACHABLE)
            })
            .map(|o| {
                o.tags
                    .iter()
                    .find(|(k, _)| k == "freemapper:source_id")
                    .map_or("", |(_, v)| v.as_str())
            })
            .collect()
    }

    /// A 522 strip `w` × `h` m from `from`, named `id`.
    fn strip(f: &Fixture, id: &str, from: (f64, f64), size: (f64, f64)) -> MapObject {
        let mut o = f.rect("522", from, size);
        o.tags.push(("freemapper:source_id".into(), id.into()));
        o
    }

    fn run(
        f: &Fixture,
        mut objects: Vec<MapObject>,
        out_of_bounds: &[MapObject],
        buildings: &[Block],
        water: &MultiPolygon<f64>,
    ) -> Vec<MapObject> {
        let footprints: Vec<Polygon<f64>> = buildings
            .iter()
            .map(|&(from, size)| f.projected_rect(from, size))
            .collect();
        hide_unreachable(
            &mut objects,
            out_of_bounds,
            &footprints,
            water,
            &f.clip,
            &f.georef,
            &f.set,
        );
        objects
    }

    /// A building: lower left corner and size in metres.
    type Block = ((f64, f64), (f64, f64));

    fn no_water() -> MultiPolygon<f64> {
        MultiPolygon(vec![])
    }

    #[test]
    fn a_passage_from_the_street_through_a_building_is_reachable() {
        let f = fixture(MapKind::Sprint);
        // A building 40 × 20 m; the passage cuts through it, its ends at the two walls.
        let found = run(
            &f,
            vec![strip(&f, "p", (10.0, 0.0), (3.0, 20.0))],
            &[],
            &[((0.0, 0.0), (40.0, 20.0))],
            &no_water(),
        );
        assert!(hidden(&found).is_empty());
    }

    #[test]
    fn a_passage_inside_a_block_that_touches_only_buildings_is_hidden() {
        let f = fixture(MapKind::Sprint);
        let found = run(
            &f,
            vec![
                strip(&f, "inside", (10.0, 5.0), (3.0, 10.0)),
                strip(&f, "through", (30.0, 0.0), (3.0, 20.0)),
            ],
            &[],
            &[((0.0, 0.0), (40.0, 20.0))],
            &no_water(),
        );
        assert_eq!(hidden(&found), ["inside"]);
    }

    #[test]
    fn the_tolerance_bridges_a_short_gap_but_not_a_long_one() {
        let f = fixture(MapKind::Sprint);
        // 2 m is the tolerance at 1:4000: a passage ending 1.5 m inside the wall is reachable, one
        // ending 3 m inside is not.
        let found = run(
            &f,
            vec![
                strip(&f, "short", (10.0, 1.5), (3.0, 17.0)),
                strip(&f, "long", (30.0, 3.0), (3.0, 14.0)),
            ],
            &[],
            &[((0.0, 0.0), (40.0, 20.0))],
            &no_water(),
        );
        assert_eq!(hidden(&found), ["long"]);
    }

    #[test]
    fn a_chain_of_passages_is_reachable_through_its_open_end() {
        let f = fixture(MapKind::Sprint);
        // Two buildings side by side and a passage in each: the first opens to the street, the
        // second continues it (they touch) and ends in a wall.
        let found = run(
            &f,
            vec![
                strip(&f, "first", (10.0, -2.0), (3.0, 10.0)),
                strip(&f, "second", (10.0, 8.0), (3.0, 6.0)),
                strip(&f, "alone", (30.0, 6.0), (3.0, 6.0)),
            ],
            &[],
            &[((0.0, 0.0), (40.0, 20.0))],
            &no_water(),
        );
        assert_eq!(hidden(&found), ["alone"]);
    }

    /// Four buildings round a courtyard of 10 × 10 m at (20, 20): the courtyard has no way out.
    fn courtyard() -> Vec<Block> {
        vec![
            ((0.0, 0.0), (40.0, 20.0)),   // south
            ((0.0, 30.0), (40.0, 20.0)),  // north
            ((0.0, 20.0), (20.0, 10.0)),  // west
            ((30.0, 20.0), (10.0, 10.0)), // east
        ]
    }

    #[test]
    fn a_passage_to_an_enclosed_courtyard_is_reachable_from_the_street() {
        let f = fixture(MapKind::Sprint);
        // Through the south building from the street into the courtyard.
        let found = run(
            &f,
            vec![strip(&f, "in", (24.0, 0.0), (3.0, 20.0))],
            &[],
            &courtyard(),
            &no_water(),
        );
        assert!(hidden(&found).is_empty());
    }

    #[test]
    fn a_passage_between_two_enclosed_courtyards_is_hidden() {
        let f = fixture(MapKind::Sprint);
        // Courtyards at (20, 20) and (50, 20), 10 × 10 m, each walled in; the passage runs through
        // the wall between them.
        let buildings = [
            ((0.0, 0.0), (70.0, 20.0)),
            ((0.0, 30.0), (70.0, 20.0)),
            ((0.0, 20.0), (20.0, 10.0)),
            ((30.0, 20.0), (20.0, 10.0)),
            ((60.0, 20.0), (10.0, 10.0)),
        ];
        let found = run(
            &f,
            vec![strip(&f, "between", (30.0, 24.0), (20.0, 3.0))],
            &[],
            &buildings,
            &no_water(),
        );
        assert_eq!(hidden(&found), ["between"]);
    }

    #[test]
    fn an_end_in_a_520_area_or_in_water_does_not_count() {
        let f = fixture(MapKind::Sprint);
        let blocked = f.rect("520", (5.0, -30.0), (33.0, 30.0));
        let found = run(
            &f,
            vec![
                strip(&f, "private", (10.0, 0.0), (3.0, 20.0)),
                strip(&f, "open", (30.0, 0.0), (3.0, 20.0)),
            ],
            &[blocked],
            &[((0.0, 0.0), (40.0, 20.0))],
            // The north end is in water too.
            &MultiPolygon(vec![f.projected_rect((5.0, 20.0), (33.0, 30.0))]),
        );
        // "private": both ends blocked (520 south, water north). "open" has its south end at
        // (30..33, 0) inside the 520 (5..38) and its north end in the water: also blocked.
        assert_eq!(hidden(&found), ["private", "open"]);
        // Move the 520 and the water away from "open".
        let blocked = f.rect("520", (5.0, -30.0), (20.0, 30.0));
        let found = run(
            &f,
            vec![strip(&f, "open", (30.0, 0.0), (3.0, 20.0))],
            &[blocked],
            &[((0.0, 0.0), (40.0, 20.0))],
            &no_water(),
        );
        assert!(hidden(&found).is_empty());
    }

    #[test]
    fn a_520_among_the_objects_counts_like_one_from_the_land_use() {
        let f = fixture(MapKind::Sprint);
        let objects = vec![
            strip(&f, "p", (10.0, 0.0), (3.0, 20.0)),
            f.rect("520", (0.0, -30.0), (40.0, 30.0)),
            f.rect("520", (0.0, 20.0), (40.0, 30.0)),
        ];
        let found = run(&f, objects, &[], &[((0.0, 0.0), (40.0, 20.0))], &no_water());
        assert_eq!(hidden(&found), ["p"]);
    }

    #[test]
    fn other_objects_stay_as_they_are_and_nothing_is_added_twice() {
        let f = fixture(MapKind::Sprint);
        let tree = f.point("418", "osm", "node/1", (20.0, 10.0), &[]);
        let wall = f.rect("515", (10.0, 5.0), (3.0, 10.0));
        let objects = vec![
            tree.clone(),
            strip(&f, "inside", (10.0, 5.0), (3.0, 10.0)),
            wall.clone(),
        ];
        let once = run(&f, objects, &[], &[((0.0, 0.0), (40.0, 20.0))], &no_water());
        assert_eq!(once[0], tree);
        assert_eq!(once[2], wall);
        let twice = run(
            &f,
            once.clone(),
            &[],
            &[((0.0, 0.0), (40.0, 20.0))],
            &no_water(),
        );
        assert_eq!(once, twice);
        assert_eq!(
            once[1].tags.iter().filter(|(k, _)| k == HIDDEN_KEY).count(),
            1
        );
    }

    #[test]
    fn no_buildings_means_everything_is_walkable() {
        let f = fixture(MapKind::Sprint);
        let found = run(
            &f,
            vec![strip(&f, "p", (10.0, 5.0), (3.0, 10.0))],
            &[],
            &[],
            &no_water(),
        );
        assert!(hidden(&found).is_empty());
    }
}
