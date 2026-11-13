//! Vegetation from the point clouds stops at the areas that shall not be entered (ADR 0105).
//!
//! ISSprOM 4.5, 520: an area that shall not be entered is olive green and shows no vegetation.
//! The forest symbol (405) is drawn in a white above the olive green, so a vegetation area from
//! the point clouds would punch a white hole into the private garden it lies in. After the
//! routing (`build/content.rs`) every vegetation area is cut at the 520 areas drawn with it: on
//! the map those of the map, in the candidates those of the map and of the candidates, as the
//! candidates PDF draws both. What is left smaller than the smallest vegetation area is dropped.

use geo::{Area as _, BooleanOps as _, BoundingRect as _, Intersects as _, MapCoords as _};
use geo_types::{MultiPolygon, Polygon};

use crate::build::content::POINT_CLOUD_KEY;
use crate::build::geometry::object_polygon;
use crate::build::out_of_bounds::area_object;
use crate::derive::vegetation::min_area_m2;
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::coord::MapCoord;
use crate::omap::{MapObject, SymbolSet};

/// The area that shall not be entered of ISSprOM.
const OUT_OF_BOUNDS_CODE: &str = "520";

/// Cuts the vegetation areas of the point clouds in `map` and `candidates` at the areas that
/// shall not be entered drawn with them. Returns how many vegetation areas were cut or dropped.
pub fn cut_vegetation_at_out_of_bounds(
    map: &mut Vec<MapObject>,
    candidates: &mut Vec<MapObject>,
    scale_denominator: u32,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<usize, GeorefError> {
    let Some(olive_id) = symbol_set.id_for_code(OUT_OF_BOUNDS_CODE) else {
        return Ok(0);
    };
    let is_vegetation = |o: &MapObject| o.tags.iter().any(|(k, _)| k == POINT_CLOUD_KEY);
    if !map.iter().chain(candidates.iter()).any(is_vegetation) {
        return Ok(0);
    }
    let projected = |polygon: Polygon<f64>| {
        polygon.map_coords(|c| {
            let p = georef.map_to_projected(MapCoord::new(c.x.round() as i32, c.y.round() as i32));
            geo_types::coord! { x: p.easting, y: p.northing }
        })
    };
    let olive_of = |list: &[MapObject]| -> Vec<Polygon<f64>> {
        list.iter()
            .filter(|o| o.symbol_id == olive_id)
            .filter_map(object_polygon)
            .map(projected)
            .collect()
    };
    let on_map = olive_of(map);
    let mut with_candidates = olive_of(candidates);
    with_candidates.extend(on_map.iter().cloned());
    let min_area = min_area_m2(scale_denominator);
    let mut changed = 0;
    for (list, olive) in [(map, on_map), (candidates, with_candidates)] {
        if olive.is_empty() || !list.iter().any(is_vegetation) {
            continue;
        }
        let mut kept = Vec::with_capacity(list.len());
        for object in list.drain(..) {
            let area = is_vegetation(&object)
                .then(|| object_polygon(&object))
                .flatten()
                .map(projected);
            let Some(area) = area else {
                kept.push(object);
                continue;
            };
            let Some(bounds) = area.bounding_rect() else {
                kept.push(object);
                continue;
            };
            let near: Vec<Polygon<f64>> = olive
                .iter()
                .filter(|o| o.bounding_rect().is_some_and(|b| b.intersects(&bounds)))
                .filter(|o| o.intersects(&area))
                .cloned()
                .collect();
            if near.is_empty() {
                kept.push(object);
                continue;
            }
            changed += 1;
            let rest = MultiPolygon::new(vec![area]).difference(&geo::unary_union(&near));
            for piece in rest.0.iter().filter(|p| p.unsigned_area() >= min_area) {
                kept.push(area_object(
                    piece,
                    object.symbol_id,
                    object.tags.clone(),
                    georef,
                )?);
            }
        }
        *list = kept;
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::points::test_support::{Fixture, fixture};
    use crate::omap::MapObjectKind;
    use crate::project::MapKind;

    fn square(f: &Fixture, code: &str, from: (f64, f64), side: f64) -> MapObject {
        let corners = [
            from,
            (from.0 + side, from.1),
            (from.0 + side, from.1 + side),
            (from.0, from.1 + side),
        ];
        let mut o = MapObject {
            symbol_id: f.id(code),
            kind: MapObjectKind::Path,
            coords: corners.iter().map(|&p| f.at(p)).collect(),
            tags: Vec::new(),
        };
        o.coords.last_mut().unwrap().flags = crate::omap::coord::FLAG_CLOSE;
        o
    }

    fn forest(f: &Fixture, from: (f64, f64), side: f64) -> MapObject {
        let mut o = square(f, "405", from, side);
        o.tags
            .push((POINT_CLOUD_KEY.to_string(), "lgb-als".to_string()));
        o
    }

    fn area_m2(f: &Fixture, o: &MapObject) -> f64 {
        let p = object_polygon(o).unwrap().map_coords(|c| {
            let p = f
                .georef
                .map_to_projected(MapCoord::new(c.x as i32, c.y as i32));
            geo_types::coord! { x: p.easting, y: p.northing }
        });
        p.unsigned_area()
    }

    fn run(f: &Fixture, map: &mut Vec<MapObject>, candidates: &mut Vec<MapObject>) -> usize {
        cut_vegetation_at_out_of_bounds(map, candidates, 4000, &f.georef, &f.set).unwrap()
    }

    #[test]
    fn a_candidate_forest_stops_at_the_candidate_garden() {
        let f = fixture(MapKind::Sprint);
        // 40 m × 40 m of forest, its eastern half in a private garden.
        let mut map = Vec::new();
        let mut candidates = vec![
            square(&f, "520", (20.0, -10.0), 60.0),
            forest(&f, (0.0, 0.0), 40.0),
        ];
        assert_eq!(run(&f, &mut map, &mut candidates), 1);
        let left: Vec<&MapObject> = candidates
            .iter()
            .filter(|o| o.symbol_id == f.id("405"))
            .collect();
        assert_eq!(left.len(), 1);
        assert!((area_m2(&f, left[0]) - 800.0).abs() < 8.0);
        assert!(left[0].tags.iter().any(|(k, _)| k == POINT_CLOUD_KEY));
        assert_eq!(
            candidates
                .iter()
                .filter(|o| o.symbol_id == f.id("520"))
                .count(),
            1
        );
    }

    #[test]
    fn a_forest_on_the_map_ignores_a_garden_that_is_only_a_candidate() {
        let f = fixture(MapKind::Sprint);
        let before = forest(&f, (0.0, 0.0), 40.0);
        let mut map = vec![before.clone()];
        let mut candidates = vec![square(&f, "520", (20.0, -10.0), 60.0)];
        assert_eq!(run(&f, &mut map, &mut candidates), 0);
        assert_eq!(map[0].coords, before.coords);
    }

    #[test]
    fn a_candidate_forest_stops_at_a_garden_on_the_map_too() {
        let f = fixture(MapKind::Sprint);
        let mut map = vec![square(&f, "520", (20.0, -10.0), 60.0)];
        let mut candidates = vec![forest(&f, (0.0, 0.0), 40.0)];
        assert_eq!(run(&f, &mut map, &mut candidates), 1);
        assert!((area_m2(&f, &candidates[0]) - 800.0).abs() < 8.0);
    }

    #[test]
    fn a_forest_inside_a_garden_goes_and_one_beside_it_stays_as_it_is() {
        let f = fixture(MapKind::Sprint);
        let beside = forest(&f, (100.0, 0.0), 30.0);
        let mut map = Vec::new();
        let mut candidates = vec![
            square(&f, "520", (0.0, 0.0), 60.0),
            forest(&f, (10.0, 10.0), 30.0),
            // A sliver of 1 m sticks out of the garden: too small to stay.
            forest(&f, (31.0, 20.0), 30.0),
            beside.clone(),
            // Park from the cadastre, no vegetation of the point clouds: not touched.
            square(&f, "401", (10.0, 10.0), 30.0),
        ];
        assert_eq!(run(&f, &mut map, &mut candidates), 2);
        let forests: Vec<&MapObject> = candidates
            .iter()
            .filter(|o| o.symbol_id == f.id("405"))
            .collect();
        assert_eq!(forests.len(), 1);
        assert_eq!(forests[0].coords, beside.coords);
        assert_eq!(
            candidates
                .iter()
                .filter(|o| o.symbol_id == f.id("401"))
                .count(),
            1
        );
    }
}
