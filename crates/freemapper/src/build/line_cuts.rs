//! Lines cut where other symbols need the room.
//!
//! The contours run through all symbols, paving, paths and buildings too (ISSprOM 2019-2, 101).
//! They are cut out only where they touch a small landform or a stairway: a small earth wall
//! (105), a small knoll (109), a small elongated knoll (110), a small depression (111), a pit or
//! hole (112), a prominent landform feature (115) and a stairway (532), as drawn on the map and
//! among the candidates ([`contour_cut_outs`]). ISOM 2017-2 breaks them at 109 and 110 and lets
//! them cross its earth wall (105): on a forest map 105 is not cut out.
//!
//! The north lines stop at the ways (ISSprOM 2019-2 §4.6): they are not drawn across paving or
//! over a path, so the way stays readable. The ways are the paved areas and the firm and unpaved
//! paths of the map and of the candidates ([`crate::build::points::way_footprints`]).
//!
//! A line is cut where it runs inside one of these footprints; stubs shorter than 1 mm on paper
//! are dropped, as along a shore.

use geo::{BooleanOps as _, BoundingRect as _, Intersects as _};
use geo_types::{Coord, LineString, MultiPolygon, Polygon};

use crate::build::conflicts::Footprints;
use crate::build::points::{HIDDEN_KEY, in_map_units, projected, stair_footprints};
use crate::build::water::outside_water;
use crate::crs::ProjectedPoint;
use crate::georef::Georeferencing;
use crate::measure::PaperMm;
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet, closed_ring};
use crate::project::MapKind;

/// The contour and the index contour.
pub const CONTOUR_CODES: [&str; 2] = ["101", "102"];
/// The magnetic north line.
pub const NORTH_LINE_CODES: [&str; 1] = ["601"];
/// Stubs of a cut line shorter than this on paper are noise.
pub const MIN_PIECE_MM: PaperMm = PaperMm(1.0);
/// The small landforms the contours are cut out at (ISSprOM 2019-2, 101): small earth wall,
/// small knoll, small elongated knoll, small depression, pit or hole, prominent landform
/// feature. The stairways come by [`stair_footprints`].
pub const CONTOUR_CUT_OUT_CODES: [&str; 6] = ["105", "109", "110", "111", "112", "115"];
/// The earth wall of ISOM 2017-2, which the contours cross (ISOM 2017-2, 2.11.3 Graphical minimum
/// dimensions, "Minimum gaps": "Crossings, such as Contours (101) and Earth walls (105, 106)"):
/// on a forest map it is no small earth wall and is not cut out.
const ISOM_EARTH_WALL_CODE: &str = "105";

/// Where the contours are cut out (ISSprOM 2019-2, 101): the drawn extent of the small
/// landforms ([`CONTOUR_CUT_OUT_CODES`]) and the footprints of the stairways among `map` and
/// `candidates`, in the map's CRS. A hidden candidate (`freemapper:hidden`) is not drawn on the
/// training map and cuts nothing.
pub fn contour_cut_outs(
    map: &[MapObject],
    candidates: &[MapObject],
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Vec<Polygon<f64>> {
    let shown = || {
        map.iter()
            .chain(candidates)
            .filter(|o| !o.tags.iter().any(|(k, _)| k == HIDDEN_KEY))
    };
    let mut cut_outs = stair_footprints(shown(), kind, georef, symbol_set);
    let ids: Vec<u32> = CONTOUR_CUT_OUT_CODES
        .iter()
        .filter(|&&code| kind != MapKind::Forest || code != ISOM_EARTH_WALL_CODE)
        .filter_map(|code| symbol_set.id_for_code(code))
        .collect();
    let landforms: Vec<&MapObject> = shown().filter(|o| ids.contains(&o.symbol_id)).collect();
    if landforms.is_empty() {
        return cut_outs;
    }
    let drawn = Footprints::of(&landforms, symbol_set);
    for i in 0..landforms.len() {
        cut_outs.extend(
            drawn
                .footprint(i)
                .0
                .iter()
                .map(|p| projected(&in_map_units(p), georef)),
        );
    }
    cut_outs
}

/// Cuts the lines among `objects` drawn with one of the symbols `codes` at `ways` (footprints
/// in the map's CRS). Pieces shorter than `min_piece_m` ground metres are dropped.
pub fn cut_lines_at_ways(
    objects: &mut Vec<MapObject>,
    codes: &[&str],
    ways: &[Polygon<f64>],
    min_piece_m: f64,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) {
    let ids: Vec<u32> = codes
        .iter()
        .filter_map(|c| symbol_set.id_for_code(c))
        .collect();
    if ways.is_empty() || ids.is_empty() {
        return;
    }
    let boxes: Vec<Option<geo_types::Rect<f64>>> = ways.iter().map(|w| w.bounding_rect()).collect();
    let mut out = Vec::with_capacity(objects.len());
    for object in objects.drain(..) {
        if !ids.contains(&object.symbol_id) || !matches!(object.kind, MapObjectKind::Path) {
            out.push(object);
            continue;
        }
        let line: LineString<f64> = object
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
        let Some(frame) = line.bounding_rect() else {
            out.push(object);
            continue;
        };
        let near: Vec<&Polygon<f64>> = ways
            .iter()
            .zip(&boxes)
            .filter(|(_, b)| b.is_some_and(|b| b.intersects(&frame)))
            .map(|(w, _)| w)
            .filter(|w| line.intersects(*w))
            .collect();
        if near.is_empty() {
            out.push(object);
            continue;
        }
        let union = near
            .iter()
            .fold(MultiPolygon::new(Vec::new()), |all, w| all.union(*w));
        for piece in outside_water(&line, &union, min_piece_m) {
            let coords: Option<Vec<(i32, i32)>> = piece
                .coords()
                .map(|c| {
                    georef
                        .projected_to_map(ProjectedPoint {
                            easting: c.x,
                            northing: c.y,
                        })
                        .ok()
                        .map(|c| (c.x, c.y))
                })
                .collect();
            let Some(coords) = coords else { continue };
            let closed = coords.len() > 3 && coords.first() == coords.last();
            out.push(MapObject {
                symbol_id: object.symbol_id,
                kind: MapObjectKind::Path,
                coords: if closed {
                    closed_ring(&coords)
                } else {
                    coords
                        .into_iter()
                        .map(|(x, y)| MapCoord::new(x, y))
                        .collect()
                },
                tags: object.tags.clone(),
            });
        }
    }
    *objects = out;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::points::test_support::{Fixture, fixture};
    use crate::project::MapKind;

    fn contour(f: &Fixture, points: &[(f64, f64)]) -> MapObject {
        MapObject {
            symbol_id: f.id("101"),
            kind: MapObjectKind::Path,
            coords: points.iter().map(|&p| f.at(p)).collect(),
            tags: vec![("freemapper:source".to_string(), "lgb-dgm1".to_string())],
        }
    }

    /// A footprint: a strip 2 m wide across x = 20 m, in the map's CRS.
    fn strip(f: &Fixture) -> Polygon<f64> {
        let (e, n) = (
            f.georef.ref_projected.easting,
            f.georef.ref_projected.northing,
        );
        let ring = vec![
            (e + 19.0, n - 50.0),
            (e + 21.0, n - 50.0),
            (e + 21.0, n + 50.0),
            (e + 19.0, n + 50.0),
            (e + 19.0, n - 50.0),
        ];
        Polygon::new(ring.into(), vec![])
    }

    #[test]
    fn contour_is_cut_at_a_footprint() {
        let f = fixture(MapKind::Sprint);
        let mut objects = vec![contour(&f, &[(0.0, 0.0), (40.0, 0.0)])];
        cut_lines_at_ways(
            &mut objects,
            &CONTOUR_CODES,
            &[strip(&f)],
            4.0,
            &f.georef,
            &f.set,
        );
        assert_eq!(objects.len(), 2, "{objects:?}");
        assert!(objects.iter().all(|o| o.tags.len() == 1));
    }

    #[test]
    fn north_line_is_cut_at_a_way() {
        let f = fixture(MapKind::Sprint);
        let mut line = contour(&f, &[(0.0, 0.0), (40.0, 0.0)]);
        line.symbol_id = f.id("601");
        let mut objects = vec![line];
        cut_lines_at_ways(
            &mut objects,
            &NORTH_LINE_CODES,
            &[strip(&f)],
            4.0,
            &f.georef,
            &f.set,
        );
        assert_eq!(objects.len(), 2);
        // The contour codes do not touch it.
        let mut objects = vec![objects[0].clone()];
        cut_lines_at_ways(
            &mut objects,
            &CONTOUR_CODES,
            &[strip(&f)],
            4.0,
            &f.georef,
            &f.set,
        );
        assert_eq!(objects.len(), 1);
    }

    #[test]
    fn contour_beside_a_footprint_stays() {
        let f = fixture(MapKind::Sprint);
        let own = contour(&f, &[(0.0, 0.0), (15.0, 0.0)]);
        let mut objects = vec![own.clone()];
        cut_lines_at_ways(
            &mut objects,
            &CONTOUR_CODES,
            &[strip(&f)],
            4.0,
            &f.georef,
            &f.set,
        );
        assert_eq!(objects, vec![own]);
    }

    #[test]
    fn the_cut_outs_are_the_shown_landforms_and_stairs() {
        let f = fixture(MapKind::Sprint);
        let knoll = |hidden: bool| MapObject {
            symbol_id: f.id("109"),
            kind: MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![f.at((0.0, 0.0))],
            tags: if hidden {
                vec![(HIDDEN_KEY.to_string(), "rock".to_string())]
            } else {
                Vec::new()
            },
        };
        let mut stair = contour(&f, &[(20.0, -5.0), (20.0, 5.0)]);
        stair.symbol_id = f.id("532.8");
        // A path is no cut-out.
        let mut path = contour(&f, &[(40.0, -5.0), (40.0, 5.0)]);
        path.symbol_id = f.id("506");
        let cut_outs = contour_cut_outs(
            &[knoll(false), path],
            &[knoll(true), stair],
            f.kind,
            &f.georef,
            &f.set,
        );
        assert_eq!(cut_outs.len(), 2, "{cut_outs:?}");
        let mut objects = vec![contour(&f, &[(-20.0, 0.0), (60.0, 0.0)])];
        cut_lines_at_ways(
            &mut objects,
            &CONTOUR_CODES,
            &cut_outs,
            4.0,
            &f.georef,
            &f.set,
        );
        assert_eq!(objects.len(), 3, "{objects:?}");
    }

    #[test]
    fn stubs_shorter_than_a_millimetre_go() {
        let f = fixture(MapKind::Sprint);
        // 2.5 m before the strip is shorter than 4 m: dropped, the rest stays.
        let mut objects = vec![contour(&f, &[(16.5, 0.0), (40.0, 0.0)])];
        cut_lines_at_ways(
            &mut objects,
            &CONTOUR_CODES,
            &[strip(&f)],
            4.0,
            &f.georef,
            &f.set,
        );
        assert_eq!(objects.len(), 1);
    }
}
