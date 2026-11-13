//! OSM firm-gravel paths as map objects (ADR 0079).
//!
//! `surface=compacted|fine_gravel` ways and `tracktype=grade2` tracks are paved by the norm
//! (ISSprOM 4.5, 501: "an area with a firm surface such as asphalt, hard gravel, tiles,
//! concrete") and become the 501.6 to 501.9 line forms, map objects of the sprint map. Two rules
//! keep them from being drawn twice or as noise:
//!
//! - Where ALKIS or the Berlin survey draws paving as a 501.5 map object, the way is the same
//!   object: the part of the line within that paving, widened by [`AREA_DEDUP_TOLERANCE_MM`] on
//!   paper, goes. A 501.5 candidate (an ALKIS `Weg` parcel) is no map object and cuts nothing.
//! - A piece shorter than [`MIN_LINE_MM`] on paper is a stub, with or without a cut, and goes.

use geo::{BooleanOps as _, Buffer as _, Euclidean, Intersects as _, Length as _, unary_union};
use geo_types::{Coord, LineString, MultiLineString, MultiPolygon};

use crate::build::candidates::MIN_LINE_MM;
use crate::build::dedup::AREA_DEDUP_TOLERANCE_MM;
use crate::build::geometry::object_polygon;
use crate::build::paved::PavedObjects;
use crate::build::points::projected;
use crate::georef::Georeferencing;
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet};

/// The line forms of a paved area (footprint 1.4, 2, 3 and 4 m) the firm-gravel ways get.
pub const FIRM_PATH_CODES: [&str; 4] = ["501.6", "501.7", "501.8", "501.9"];
/// ISSprOM 501.5 paved area, light traffic: the ALKIS and survey paving.
const PAVING_CODE: &str = "501.5";

/// `objects` with the OSM firm-gravel lines ([`FIRM_PATH_CODES`]) cut where `paved.map` (501.5
/// map objects; the candidates are ignored) lies and without the stubs. Everything else stays as it was.
pub fn cut_firm_paths(
    objects: Vec<MapObject>,
    paved: &PavedObjects,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Vec<MapObject> {
    // A set that answers these codes through aliases (ISOM) draws ordinary paths and roads with
    // the same symbols; it has no firm-gravel forms to cut.
    let firm: Vec<u32> = FIRM_PATH_CODES
        .iter()
        .filter(|code| !symbol_set.is_aliased(code))
        .filter_map(|code| symbol_set.id_for_code(code))
        .collect();
    let is_firm = |o: &MapObject| {
        firm.contains(&o.symbol_id)
            && matches!(o.kind, MapObjectKind::Path)
            && o.coords.len() >= 2
            && o.tags
                .iter()
                .any(|(k, v)| k == "freemapper:source" && v == "osm")
    };
    if !objects.iter().any(is_firm) {
        return objects;
    }
    let metres_per_mm = f64::from(georef.scale_denominator) / 1000.0;
    let min_length = MIN_LINE_MM * metres_per_mm;
    let paving = paving_union(
        &paved.map,
        georef,
        symbol_set,
        AREA_DEDUP_TOLERANCE_MM * metres_per_mm,
    );
    let mut kept = Vec::with_capacity(objects.len());
    for object in objects {
        if !is_firm(&object) {
            kept.push(object);
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
        let touches = !paving.0.is_empty() && line.intersects(&paving);
        if !touches {
            if Euclidean.length(&line) >= min_length {
                kept.push(object);
            }
            continue;
        }
        let pieces = paving.clip(&MultiLineString(vec![line]), true);
        for piece in pieces.0 {
            if piece.0.len() < 2 || Euclidean.length(&piece) < min_length {
                continue;
            }
            let coords: Option<Vec<MapCoord>> = piece
                .coords()
                .map(|c| {
                    georef
                        .projected_to_map(crate::crs::ProjectedPoint {
                            easting: c.x,
                            northing: c.y,
                        })
                        .ok()
                })
                .collect();
            if let Some(coords) = coords {
                kept.push(MapObject {
                    symbol_id: object.symbol_id,
                    kind: MapObjectKind::Path,
                    coords,
                    tags: object.tags.clone(),
                });
            }
        }
    }
    kept
}

/// The 501.5 areas among `paving` as one polygon set in the map's CRS, widened by `tolerance_m`.
fn paving_union(
    paving: &[MapObject],
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
    tolerance_m: f64,
) -> MultiPolygon<f64> {
    let Some(id) = symbol_set.id_for_code(PAVING_CODE) else {
        return MultiPolygon(Vec::new());
    };
    let polygons: Vec<_> = paving
        .iter()
        .filter(|o| o.symbol_id == id)
        .filter_map(object_polygon)
        .map(|p| projected(&p, georef))
        .collect();
    if polygons.is_empty() {
        return MultiPolygon(Vec::new());
    }
    unary_union(&polygons).buffer(tolerance_m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::points::test_support::{Fixture, fixture};
    use crate::project::MapKind;

    fn osm_line(f: &Fixture, code: &str, id: &str, from: (f64, f64), to: (f64, f64)) -> MapObject {
        MapObject {
            symbol_id: f.id(code),
            kind: MapObjectKind::Path,
            coords: vec![f.at(from), f.at(to)],
            tags: vec![
                ("freemapper:source".into(), "osm".into()),
                ("freemapper:source_id".into(), id.into()),
            ],
        }
    }

    fn map_paving(map: Vec<MapObject>) -> PavedObjects {
        PavedObjects {
            map,
            candidates: Vec::new(),
        }
    }

    fn lengths_m(f: &Fixture, objects: &[MapObject]) -> Vec<(String, f64)> {
        objects
            .iter()
            .map(|o| {
                let id = o
                    .tags
                    .iter()
                    .find(|(k, _)| k == "freemapper:source_id")
                    .map_or(String::new(), |(_, v)| v.clone());
                let m: f64 = o
                    .coords
                    .windows(2)
                    .map(|w| {
                        f64::from(w[1].x - w[0].x).hypot(f64::from(w[1].y - w[0].y)) / f.metre()
                    })
                    .sum();
                (id, m)
            })
            .collect()
    }

    #[test]
    fn a_path_in_the_open_stays_whole() {
        let f = fixture(MapKind::Sprint);
        let line = osm_line(&f, "501.6", "way/1", (0.0, 0.0), (60.0, 0.0));
        let out = cut_firm_paths(
            vec![line.clone()],
            &PavedObjects::default(),
            &f.georef,
            &f.set,
        );
        assert_eq!(out, vec![line]);
    }

    #[test]
    fn the_part_in_alkis_paving_goes() {
        let f = fixture(MapKind::Sprint);
        // A 100 m way; ALKIS paving covers x 40 to 100 m, 10 m wide round the way.
        let line = osm_line(&f, "501.7", "way/1", (0.0, 0.0), (100.0, 0.0));
        let street = f.rect("501.5", (40.0, -5.0), (80.0, 10.0));
        let out = cut_firm_paths(vec![line], &map_paving(vec![street]), &f.georef, &f.set);
        let lengths = lengths_m(&f, &out);
        assert_eq!(lengths.len(), 1, "{lengths:?}");
        // 40 m, less the 1 m tolerance on paper (0.25 mm at 1:4000).
        assert!((lengths[0].1 - 39.0).abs() < 0.2, "{lengths:?}");
    }

    #[test]
    fn a_path_wholly_in_paving_goes() {
        let f = fixture(MapKind::Sprint);
        let line = osm_line(&f, "501.6", "way/1", (10.0, 0.0), (50.0, 0.0));
        let square = f.rect("501.5", (0.0, -10.0), (60.0, 20.0));
        assert!(
            cut_firm_paths(vec![line], &map_paving(vec![square]), &f.georef, &f.set).is_empty()
        );
    }

    #[test]
    fn a_path_that_leaves_paving_in_the_middle_keeps_both_ends() {
        let f = fixture(MapKind::Sprint);
        let line = osm_line(&f, "501.6", "way/1", (0.0, 0.0), (100.0, 0.0));
        let square = f.rect("501.5", (40.0, -10.0), (20.0, 20.0));
        let out = cut_firm_paths(vec![line], &map_paving(vec![square]), &f.georef, &f.set);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn a_stub_shorter_than_1_mm_on_paper_goes() {
        let f = fixture(MapKind::Sprint);
        // 1 mm at 1:4000 is 4 m.
        let stub = osm_line(&f, "501.6", "way/1", (0.0, 0.0), (3.0, 0.0));
        let ok = osm_line(&f, "501.6", "way/2", (0.0, 20.0), (5.0, 20.0));
        let out = cut_firm_paths(vec![stub, ok], &PavedObjects::default(), &f.georef, &f.set);
        assert_eq!(lengths_m(&f, &out).len(), 1);
        assert_eq!(lengths_m(&f, &out)[0].0, "way/2");
    }

    /// On a forest map 505 and 503 are the OSM footpaths and roads, not the firm-gravel forms the
    /// ISSprOM codes alias to: a short piece of a split way stays.
    #[test]
    fn short_osm_paths_stay_on_a_forest_map() {
        let f = fixture(MapKind::Forest);
        // 1 mm at 1:15000 is 15 m.
        let footpath = osm_line(&f, "505", "way/1", (0.0, 0.0), (5.0, 0.0));
        let road = osm_line(&f, "503", "way/2", (0.0, 20.0), (5.0, 20.0));
        let out = cut_firm_paths(
            vec![footpath, road],
            &PavedObjects::default(),
            &f.georef,
            &f.set,
        );
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn a_remnant_stub_beside_paving_goes() {
        let f = fixture(MapKind::Sprint);
        // 41 m way: 38 m in paving less the tolerance leaves a 2 m stub.
        let line = osm_line(&f, "501.6", "way/1", (0.0, 0.0), (41.0, 0.0));
        let square = f.rect("501.5", (3.0, -10.0), (60.0, 20.0));
        let out = cut_firm_paths(vec![line], &map_paving(vec![square]), &f.georef, &f.set);
        assert!(out.is_empty(), "{:?}", lengths_m(&f, &out));
    }

    #[test]
    fn other_lines_and_other_paving_do_not_matter() {
        let f = fixture(MapKind::Sprint);
        let unpaved = osm_line(&f, "505.1", "way/1", (0.0, 0.0), (60.0, 0.0));
        let mut cadastre = osm_line(&f, "501.6", "way/2", (0.0, 5.0), (60.0, 5.0));
        cadastre.tags[0].1 = "alkis".into();
        let green = f.rect("401", (0.0, -10.0), (60.0, 20.0));
        let paving_candidate_only_is_not_a_401 = map_paving(vec![green]);
        let out = cut_firm_paths(
            vec![unpaved.clone(), cadastre.clone()],
            &paving_candidate_only_is_not_a_401,
            &f.georef,
            &f.set,
        );
        assert_eq!(out, vec![unpaved, cadastre]);
    }

    #[test]
    fn a_weg_candidate_parcel_keeps_the_line_whole() {
        let f = fixture(MapKind::Sprint);
        let line = osm_line(&f, "501.7", "way/1", (0.0, 0.0), (100.0, 0.0));
        let weg = f.rect("501.5", (-10.0, -5.0), (120.0, 10.0));
        let paved = PavedObjects {
            map: Vec::new(),
            candidates: vec![weg],
        };
        let out = cut_firm_paths(vec![line.clone()], &paved, &f.georef, &f.set);
        assert_eq!(out, vec![line]);
    }

    #[test]
    fn a_candidate_parcel_does_not_stop_the_stub_rule() {
        let f = fixture(MapKind::Sprint);
        let stub = osm_line(&f, "501.6", "way/1", (0.0, 0.0), (3.0, 0.0));
        let weg = f.rect("501.5", (-10.0, -5.0), (20.0, 10.0));
        let paved = PavedObjects {
            map: Vec::new(),
            candidates: vec![weg],
        };
        assert!(cut_firm_paths(vec![stub], &paved, &f.georef, &f.set).is_empty());
    }

    #[test]
    fn only_the_map_object_paving_cuts_when_both_overlap() {
        let f = fixture(MapKind::Sprint);
        // A 100 m way; a street (map object) covers x 40 to 100 m, a Weg parcel x 0 to 40 m.
        let line = osm_line(&f, "501.7", "way/1", (0.0, 0.0), (100.0, 0.0));
        let street = f.rect("501.5", (40.0, -5.0), (80.0, 10.0));
        let weg = f.rect("501.5", (0.0, -5.0), (40.0, 10.0));
        let paved = PavedObjects {
            map: vec![street],
            candidates: vec![weg],
        };
        let out = cut_firm_paths(vec![line], &paved, &f.georef, &f.set);
        let lengths = lengths_m(&f, &out);
        assert_eq!(lengths.len(), 1, "{lengths:?}");
        assert!((lengths[0].1 - 39.0).abs() < 0.2, "{lengths:?}");
    }
}
