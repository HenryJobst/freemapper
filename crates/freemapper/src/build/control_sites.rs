//! Control sites for course setters (ADR 0074): the street furniture of OSM that a sprint map
//! does not draw as a map object but that makes a good control location, as 531 candidates.
//! A site must be unambiguous: where a second object of the same kind stands within
//! [`SAME_KIND_M`], neither is kept. A row of bollards, a pair of benches or the lamps along a
//! street cannot be told apart on the ground. The 0.15 mm gap between point symbols is kept by
//! `build/points.rs`, where a control site also gives way to every other symbol.

use std::collections::HashMap;

use crate::omap::{MapObject, MapObjectKind};

/// The tag that names the kind of a control site (`bollard`, `street_lamp`, `waste_basket`,
/// `bicycle_parking`, `bench`). A course planner reads the control sites from the candidates by
/// this tag.
pub const CONTROL_KEY: &str = "freemapper:control";

/// Two control sites of one kind closer than this (ground metres) are a row, a pair or a series,
/// not a site. Neither the norm nor the IOF competition rules give a figure for the distance
/// between controls on identical features; 25 m is 6 mm on a 1:4000 sheet,
/// and keeps a bollard row (posts 1 to 5 m apart) and a street lamp series (20 to 25 m) out. A
/// ruling (2026-11-10).
pub const SAME_KIND_M: f64 = 25.0;

fn tag<'a>(object: &'a MapObject, key: &str) -> Option<&'a str> {
    object
        .tags
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

/// `objects` without the control sites that have a site of the same kind within [`SAME_KIND_M`].
/// Other objects, and all objects of a map without control sites, pass unchanged and in order.
/// `scale_denominator` turns map units (1/1000 mm on paper) into ground metres.
pub fn drop_ambiguous(objects: Vec<MapObject>, scale_denominator: u32) -> Vec<MapObject> {
    let metre = 1e6 / f64::from(scale_denominator);
    let reach = SAME_KIND_M * metre;
    let sites: Vec<(usize, &str, (f64, f64))> = objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            let kind = tag(o, CONTROL_KEY)?;
            let MapObjectKind::Point { .. } = o.kind else {
                return None;
            };
            let c = o.coords.first()?;
            Some((i, kind, (f64::from(c.x), f64::from(c.y))))
        })
        .collect();
    let cell = |(x, y): (f64, f64)| ((x / reach).floor() as i64, (y / reach).floor() as i64);
    let mut grid: HashMap<(&str, i64, i64), Vec<usize>> = HashMap::new();
    for (n, &(_, kind, xy)) in sites.iter().enumerate() {
        let (cx, cy) = cell(xy);
        grid.entry((kind, cx, cy)).or_default().push(n);
    }
    let mut crowded = vec![false; objects.len()];
    for (n, &(i, kind, xy)) in sites.iter().enumerate() {
        let (cx, cy) = cell(xy);
        crowded[i] = (cx - 1..=cx + 1)
            .flat_map(|x| (cy - 1..=cy + 1).map(move |y| (x, y)))
            .flat_map(|(x, y)| grid.get(&(kind, x, y)).into_iter().flatten())
            .any(|&m| {
                let other = sites[m].2;
                m != n && (xy.0 - other.0).hypot(xy.1 - other.1) < reach
            });
    }
    objects
        .into_iter()
        .enumerate()
        .filter(|(i, _)| !crowded[*i])
        .map(|(_, o)| o)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::omap::MapCoord;

    /// Map units per ground metre at 1:4000.
    const M: f64 = 250.0;

    fn site(kind: Option<&str>, id: &str, east_m: f64, north_m: f64) -> MapObject {
        let mut tags = vec![("freemapper:source_id".to_string(), id.to_string())];
        if let Some(kind) = kind {
            tags.push((CONTROL_KEY.to_string(), kind.to_string()));
        }
        MapObject {
            symbol_id: 1,
            kind: MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![MapCoord::new((east_m * M) as i32, (north_m * M) as i32)],
            tags,
        }
    }

    fn ids(objects: &[MapObject]) -> Vec<&str> {
        objects
            .iter()
            .map(|o| tag(o, "freemapper:source_id").unwrap())
            .collect()
    }

    #[test]
    fn a_row_of_bollards_is_dropped_and_an_isolated_one_stays() {
        let objects = vec![
            site(Some("bollard"), "a", 0.0, 0.0),
            site(Some("bollard"), "b", 3.0, 0.0),
            site(Some("bollard"), "c", 6.0, 0.0),
            site(Some("bollard"), "far", 60.0, 0.0),
        ];
        assert_eq!(ids(&drop_ambiguous(objects, 4000)), ["far"]);
    }

    #[test]
    fn the_limit_is_twenty_five_metres() {
        let near = vec![
            site(Some("bench"), "a", 0.0, 0.0),
            site(Some("bench"), "b", 24.9, 0.0),
        ];
        assert!(drop_ambiguous(near, 4000).is_empty());
        let apart = vec![
            site(Some("bench"), "a", 0.0, 0.0),
            site(Some("bench"), "b", 25.1, 0.0),
        ];
        assert_eq!(ids(&drop_ambiguous(apart, 4000)), ["a", "b"]);
    }

    #[test]
    fn other_kinds_and_other_objects_do_not_count() {
        let objects = vec![
            site(Some("bollard"), "bollard", 0.0, 0.0),
            site(Some("street_lamp"), "lamp", 1.0, 0.0),
            site(None, "tree", 2.0, 0.0),
            site(None, "tree2", 2.5, 0.0),
        ];
        assert_eq!(
            ids(&drop_ambiguous(objects, 4000)),
            ["bollard", "lamp", "tree", "tree2"]
        );
    }

    #[test]
    fn the_distance_is_ground_metres_at_any_scale() {
        // 6 m apart at 1:4000 is 6 m apart at 1:5000 too: the map units shrink, and 30 m is
        // 6 mm there.
        let mut a = site(Some("bollard"), "a", 0.0, 0.0);
        let mut b = site(Some("bollard"), "b", 0.0, 0.0);
        a.coords = vec![MapCoord::new(0, 0)];
        // 6 m at 1:5000 is 1.2 mm on paper = 1200 map units.
        b.coords = vec![MapCoord::new(1200, 0)];
        assert!(drop_ambiguous(vec![a.clone(), b], 5000).is_empty());
        let mut c = site(Some("bollard"), "c", 0.0, 0.0);
        c.coords = vec![MapCoord::new(6000, 0)];
        assert_eq!(drop_ambiguous(vec![a, c], 5000).len(), 2);
    }

    #[test]
    fn the_order_of_what_stays_is_kept() {
        let objects = vec![
            site(Some("bench"), "z", 100.0, 0.0),
            site(None, "tree", 0.0, 0.0),
            site(Some("bench"), "a", 50.0, 0.0),
        ];
        assert_eq!(ids(&drop_ambiguous(objects, 4000)), ["z", "tree", "a"]);
    }
}
