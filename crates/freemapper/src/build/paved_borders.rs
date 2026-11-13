//! The border of paved areas (ISSprOM 2019-2 §4.5, 501.1).
//!
//! "A paved area ... should be bordered (or framed) by the symbol Step or edge of paved area
//! (501.1)." The 501.5 areas of the ALKIS traffic parcels, the street survey and the land use
//! come without one: a parcel boundary is no visible edge, and ADR 0069 did not draw one. Here
//! every 501.5 area gets the outline of its union as 501.1, minus what already shows an edge: a
//! building outline, water, a 501.1 line that is there (the street survey's kerbs) and the edge
//! of the map. The areas of the map and the areas among the candidates are bordered apart, each
//! border going to the list of its areas; a candidate edge along a map edge is not drawn twice.
//! A border piece shorter than 1 mm on paper is left out.

use geo::{BooleanOps as _, Buffer as _, unary_union};
use geo_types::{LineString, MultiPolygon, Polygon, Rect};

use crate::build::geometry::object_polygon;
use crate::build::points::{GENERALIZATION_SOURCE, boundary_lines, line_coords, projected};
use crate::crs::BoundingBox;
use crate::georef::Georeferencing;
use crate::measure::PaperMm;
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet};

/// The paved area, light traffic, and the edge of a paved area.
const PAVED_CODE: &str = "501.5";
const EDGE_CODE: &str = "501.1";
/// A building outline, water and the edge of the map already show an edge within this distance
/// (ground metres).
const KNOWN_EDGE_M: f64 = 0.4;
/// A 501.1 line that is there covers an edge within this distance (ground metres): 0.2 mm on
/// paper at 1:4000, so a kerb and a border that differ by a hair are not drawn twice.
const EDGE_LINE_M: f64 = 0.75;
/// A border shorter than this on paper is not drawn.
const MIN_BORDER_MM: PaperMm = PaperMm(1.0);
/// A border is simplified by this much (ground metres).
const SIMPLIFY_M: f64 = 0.05;
/// The `freemapper:generalized` value of a border line.
const BORDER: &str = "border of a paved area (ISSprOM 501)";

/// Adds the 501.1 borders of the 501.5 areas among `lists` (the map first, the candidates
/// second): the border of a map area to the map, that of a candidate area to the candidates.
/// `footprints` and `water` are in the map's CRS, so is the edge of the map `clip`.
pub fn border_paved_areas(
    lists: &mut [&mut Vec<MapObject>],
    footprints: &[Polygon<f64>],
    water: &MultiPolygon<f64>,
    clip: &BoundingBox,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> usize {
    let (Some(paved), Some(edge)) = (
        symbol_set.id_for_code(PAVED_CODE),
        symbol_set.id_for_code(EDGE_CODE),
    ) else {
        return 0;
    };
    let min_length = MIN_BORDER_MM.metres(georef.scale_denominator);
    let areas = |list: &Vec<MapObject>| -> Vec<Polygon<f64>> {
        list.iter()
            .filter(|o| o.symbol_id == paved)
            .filter_map(object_polygon)
            .map(|p| projected(&p, georef))
            .collect()
    };
    let edge_lines: Vec<LineString<f64>> = lists
        .iter()
        .flat_map(|l| l.iter())
        .filter(|o| o.symbol_id == edge && matches!(o.kind, MapObjectKind::Path))
        .map(|o| {
            o.coords
                .iter()
                .map(|&c| {
                    let p = georef.map_to_projected(c);
                    geo_types::Coord {
                        x: p.easting,
                        y: p.northing,
                    }
                })
                .collect()
        })
        .collect();

    // What shows an edge already.
    let mut known: Vec<Polygon<f64>> = Vec::new();
    known.extend(footprints.iter().flat_map(|p| p.buffer(KNOWN_EDGE_M).0));
    known.extend(water.buffer(KNOWN_EDGE_M).0);
    known.extend(edge_lines.iter().flat_map(|l| l.buffer(EDGE_LINE_M).0));
    // The edge of the map: the ring between the clip rectangle and the one 0.6 m inside it.
    let frame = Rect::new(
        geo_types::coord! { x: clip.min_e, y: clip.min_n },
        geo_types::coord! { x: clip.max_e, y: clip.max_n },
    )
    .to_polygon();
    let inner = Rect::new(
        geo_types::coord! { x: clip.min_e + 0.6, y: clip.min_n + 0.6 },
        geo_types::coord! { x: clip.max_e - 0.6, y: clip.max_n - 0.6 },
    )
    .to_polygon();
    known.extend(frame.difference(&inner).0);
    let mut known = unary_union(&known);

    let mut added = 0;
    for list in lists.iter_mut() {
        let union = unary_union(&areas(list));
        let mut lines: Vec<LineString<f64>> = Vec::new();
        for polygon in &union.0 {
            lines.extend(boundary_lines(polygon, &known, min_length, SIMPLIFY_M));
        }
        let borders: Vec<MapObject> = lines
            .iter()
            .filter_map(|line| line_coords(line, georef).ok().map(|coords| (line, coords)))
            .map(|(_, coords)| MapObject {
                symbol_id: edge,
                kind: MapObjectKind::Path,
                coords: coords
                    .into_iter()
                    .map(|c: MapCoord| MapCoord::new(c.x, c.y))
                    .collect(),
                tags: vec![
                    (
                        "freemapper:source".to_string(),
                        GENERALIZATION_SOURCE.to_string(),
                    ),
                    ("freemapper:generalized".to_string(), BORDER.to_string()),
                ],
            })
            .collect();
        // The next list does not repeat an edge of this one.
        let drawn: Vec<Polygon<f64>> = lines.iter().flat_map(|x| x.buffer(EDGE_LINE_M).0).collect();
        if !drawn.is_empty() {
            known = known.union(&unary_union(&drawn));
        }
        added += borders.len();
        list.extend(borders);
    }
    added
}

/// What [`border_paved_areas`] reads of `lists`: per list, its 501.5 areas and its 501.1 lines
/// in list order. It reads nothing else of them (the areas and the edge lines are picked by
/// symbol) and only appends to them, so it adds the same borders to these lists as to the whole
/// ones. A border computed for one input fits every placement with an equal input.
pub fn border_input(lists: &[&Vec<MapObject>], symbol_set: &SymbolSet) -> Vec<Vec<MapObject>> {
    let paved = symbol_set.id_for_code(PAVED_CODE);
    let edge = symbol_set.id_for_code(EDGE_CODE);
    lists
        .iter()
        .map(|list| {
            list.iter()
                .filter(|o| {
                    Some(o.symbol_id) == paved
                        || (Some(o.symbol_id) == edge && matches!(o.kind, MapObjectKind::Path))
                })
                .cloned()
                .collect()
        })
        .collect()
}

/// The borders [`border_paved_areas`] adds to lists whose [`border_input`] is `input`: per list,
/// the objects to append to it.
pub fn paved_borders(
    input: &[Vec<MapObject>],
    footprints: &[Polygon<f64>],
    water: &MultiPolygon<f64>,
    clip: &BoundingBox,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Vec<Vec<MapObject>> {
    let mut lists = input.to_vec();
    border_paved_areas(
        &mut lists.iter_mut().collect::<Vec<_>>(),
        footprints,
        water,
        clip,
        georef,
        symbol_set,
    );
    lists
        .iter_mut()
        .zip(input)
        .map(|(list, read)| list.split_off(read.len()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::points::test_support::{Fixture, fixture};
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

    fn edge_count(f: &Fixture, list: &[MapObject]) -> usize {
        list.iter().filter(|o| o.symbol_id == f.id("501.1")).count()
    }

    fn run(
        f: &Fixture,
        map: &mut Vec<MapObject>,
        candidates: &mut Vec<MapObject>,
        footprints: &[Polygon<f64>],
    ) -> usize {
        border_paved_areas(
            &mut [map, candidates],
            footprints,
            &MultiPolygon::new(Vec::new()),
            &f.clip,
            &f.georef,
            &f.set,
        )
    }

    /// A rectangle (east, north from the reference point, metres) in the map's CRS.
    fn rect(f: &Fixture, from: (f64, f64), size: (f64, f64)) -> Polygon<f64> {
        let (e, n) = (
            f.georef.ref_projected.easting,
            f.georef.ref_projected.northing,
        );
        Rect::new(
            geo_types::coord! { x: e + from.0, y: n + from.1 },
            geo_types::coord! { x: e + from.0 + size.0, y: n + from.1 + size.1 },
        )
        .to_polygon()
    }

    #[test]
    fn a_paved_area_gets_its_border() {
        let f = fixture(MapKind::Sprint);
        let mut map = vec![square(&f, "501.5", (0.0, 0.0), 40.0)];
        let mut candidates = Vec::new();
        assert!(run(&f, &mut map, &mut candidates, &[]) >= 1);
        assert!(edge_count(&f, &map) >= 1);
        assert_eq!(edge_count(&f, &candidates), 0);
        let border = map.iter().find(|o| o.symbol_id == f.id("501.1")).unwrap();
        assert!(
            border
                .tags
                .iter()
                .any(|(k, v)| k == "freemapper:source" && v == "generalization")
        );
    }

    #[test]
    fn a_building_side_has_no_border() {
        let f = fixture(MapKind::Sprint);
        let mut map = vec![square(&f, "501.5", (0.0, 0.0), 40.0)];
        // A building along the whole south side: its outline is the edge there.
        let footprints = [rect(&f, (-10.0, -10.0), (60.0, 10.0))];
        let mut with = (map.clone(), Vec::new());
        let mut without = (map.clone(), Vec::new());
        run(&f, &mut without.0, &mut without.1, &[]);
        run(&f, &mut with.0, &mut with.1, &footprints);
        let length = |list: &[MapObject]| -> usize {
            list.iter()
                .filter(|o| o.symbol_id == f.id("501.1"))
                .map(|o| o.coords.len())
                .sum()
        };
        assert!(length(&with.0) < length(&without.0), "{map:?}");
        map.clear();
    }

    #[test]
    fn an_existing_edge_is_not_drawn_twice() {
        let f = fixture(MapKind::Sprint);
        let mut kerb = square(&f, "501.1", (0.0, 0.0), 40.0);
        kerb.coords.last_mut().unwrap().flags = 0;
        // A kerb line all round: back to the first corner.
        kerb.coords.push(kerb.coords[0]);
        let mut map = vec![square(&f, "501.5", (0.0, 0.0), 40.0), kerb];
        let mut candidates = Vec::new();
        let added = run(&f, &mut map, &mut candidates, &[]);
        assert_eq!(added, 0, "{map:?}");
    }

    #[test]
    fn candidate_areas_are_bordered_in_the_candidates() {
        let f = fixture(MapKind::Sprint);
        let mut map = Vec::new();
        let mut candidates = vec![square(&f, "501.5", (0.0, 0.0), 40.0)];
        run(&f, &mut map, &mut candidates, &[]);
        assert_eq!(edge_count(&f, &map), 0);
        assert!(edge_count(&f, &candidates) >= 1);
    }

    #[test]
    fn borders_from_the_input_are_those_of_the_whole_lists() {
        let f = fixture(MapKind::Sprint);
        let mut kerb = square(&f, "501.1", (0.0, 0.0), 20.0);
        kerb.coords.last_mut().unwrap().flags = 0;
        let mut map = vec![
            square(&f, "501.5", (0.0, 0.0), 40.0),
            square(&f, "518", (5.0, 5.0), 10.0),
            kerb,
        ];
        let mut candidates = vec![
            square(&f, "410.1", (50.0, 0.0), 10.0),
            square(&f, "501.5", (60.0, 0.0), 30.0),
        ];
        let input = border_input(&[&map, &candidates], &f.set);
        assert_eq!(input[0].len(), 2);
        assert_eq!(input[1].len(), 1);
        let added = paved_borders(
            &input,
            &[],
            &MultiPolygon::new(Vec::new()),
            &f.clip,
            &f.georef,
            &f.set,
        );
        let (map_len, candidates_len) = (map.len(), candidates.len());
        run(&f, &mut map, &mut candidates, &[]);
        assert!(!added[0].is_empty() && !added[1].is_empty());
        assert_eq!(map[map_len..], added[0][..]);
        assert_eq!(candidates[candidates_len..], added[1][..]);
    }

    #[test]
    fn a_candidate_edge_along_a_map_edge_is_not_repeated() {
        let f = fixture(MapKind::Sprint);
        let mut map = vec![square(&f, "501.5", (0.0, 0.0), 40.0)];
        let mut candidates = vec![square(&f, "501.5", (0.0, 0.0), 40.0)];
        run(&f, &mut map, &mut candidates, &[]);
        assert!(edge_count(&f, &map) >= 1);
        assert_eq!(edge_count(&f, &candidates), 0, "{candidates:?}");
    }
}
