// SPDX-License-Identifier: AGPL-3.0-or-later

//! Carriageways (spec 2026-11-14, "The second half": Carriageways; ISSprOM 2019-2, 501: "Edges
//! within paved areas are generally not represented, unless they serve navigation", "Differences
//! in the brown colour shall be used to show differences in the traffic").
//!
//! Only Berlin has a carriageway in the data: the street survey's `cm_fahrbahn`, which the build
//! passes inside the street parcels as hidden 501.11 candidates ([`HIDDEN_CARRIAGEWAY`]). On the
//! training map each is drawn, cut to the 501.5 street area it lies in, and the survey's kerb
//! lines inside a street area go; the generated border of the street area stays. Elsewhere there
//! is neither and nothing changes.

use geo::{Area as _, BooleanOps as _, BoundingRect as _, Buffer as _, Intersects as _};
use geo_types::{LineString, MultiLineString, MultiPolygon, Polygon, Rect};

use crate::build::conflicts::TOLERANCE_MM;
use crate::build::geometry::object_polygon;
use crate::build::out_of_bounds::area_object;
use crate::build::paved::{CARRIAGEWAY_CODE, HIDDEN_CARRIAGEWAY, STREET_NUTZARTEN};
use crate::build::paved_borders::EDGE_LINE_M;
use crate::build::points::{HIDDEN_KEY, projected};
use crate::georef::Georeferencing;
use crate::measure::{PaperMm, PaperMm2, SymbolScale, map_units_per_metre};
use crate::omap::{MapObject, MapObjectKind, SymbolSet};
use crate::project::MapKind;
use crate::render::geometry::parts;

use super::{OPEN_LAND_MIN_FOREST, OPEN_LAND_MIN_SPRINT};

/// The street area: 501.5 paved area, light traffic.
const STREET_CODE: &str = "501.5";
/// The edge of a paved area, the symbol of the survey's kerb lines.
const EDGE_CODE: &str = "501.1";
/// The `freemapper:source` of a kerb line of the street survey.
const KERB_SOURCE: &str = "geoportal-berlin/sb-bordstein";

/// What [`draw_carriageways`] did.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct Carriageways {
    /// The 501.11 objects drawn.
    pub drawn: usize,
    /// The kerb lines that went.
    pub kerbs: usize,
}

/// The carriageways of `objects` (501.11 with [`HIDDEN_KEY`]` = `[`HIDDEN_CARRIAGEWAY`]) are
/// drawn: the hidden tag goes, each is cut to the union of the street areas (501.5 whose
/// `freemapper:nutzart` names one of [`STREET_NUTZARTEN`]) and pieces below 501.11's minimum
/// area ([`min_area`]) go. A kerb line of the survey (501.1, [`KERB_SOURCE`]) goes when more than
/// half of it lies inside a street area shrunk by [`EDGE_LINE_M`]: a kerb within that of the
/// street area's edge is the edge the generated border (`freemapper:source=generalization`)
/// leaves to it (`build/paved_borders.rs`), so it stays, as the border does. A hidden carriageway
/// is never left hidden: without a street area or a 501.11 symbol it goes.
pub(super) fn draw_carriageways(
    objects: &mut Vec<MapObject>,
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Carriageways {
    let is_carriageway = |o: &MapObject| {
        o.tags
            .iter()
            .any(|(k, v)| k == HIDDEN_KEY && v == HIDDEN_CARRIAGEWAY)
    };
    let (heavy, light) = (
        symbol_set.id_for_code(CARRIAGEWAY_CODE),
        symbol_set.id_for_code(STREET_CODE),
    );
    let streets: Vec<(Rect<f64>, Polygon<f64>)> = objects
        .iter()
        .filter(|o| Some(o.symbol_id) == light && is_street(o))
        .filter_map(object_polygon)
        .filter_map(|p| Some((p.bounding_rect()?, p)))
        .collect();
    let min_m2 = heavy.map_or(0.0, |id| min_area(id, kind, georef, symbol_set));
    let mut report = Carriageways::default();
    let before = std::mem::take(objects);
    for object in before {
        if !is_carriageway(&object) {
            objects.push(object);
            continue;
        }
        let Some(heavy) = heavy else {
            continue;
        };
        let pieces = object_polygon(&object)
            .map(|p| inside_streets(&p, &streets))
            .unwrap_or_default();
        let mut tags = object.tags.clone();
        tags.retain(|(k, _)| k != HIDDEN_KEY);
        for piece in pieces {
            let area_m2 =
                PaperMm2(piece.unsigned_area() / 1e6).square_metres(georef.scale_denominator);
            if area_m2 < min_m2 {
                continue;
            }
            if let Ok(drawn) = area_object(&projected(&piece, georef), heavy, tags.clone(), georef)
            {
                objects.push(drawn);
                report.drawn += 1;
            }
        }
    }
    if let Some(edge) = symbol_set.id_for_code(EDGE_CODE)
        && !streets.is_empty()
    {
        let shrink = EDGE_LINE_M * map_units_per_metre(georef.scale_denominator);
        let inner: Vec<(Rect<f64>, Polygon<f64>)> =
            geo::unary_union(streets.iter().map(|(_, p)| p))
                .buffer(-shrink)
                .0
                .into_iter()
                .filter_map(|p| Some((p.bounding_rect()?, p)))
                .collect();
        let before = objects.len();
        objects.retain(|o| {
            let kerb = o.symbol_id == edge
                && matches!(o.kind, MapObjectKind::Path)
                && o.tags
                    .iter()
                    .any(|(k, v)| k == "freemapper:source" && v == KERB_SOURCE);
            !(kerb && mostly_inside(o, &inner))
        });
        report.kerbs = before - objects.len();
    }
    report
}

/// Whether the 501.5 `object` is a street area: its `freemapper:nutzart` is one of
/// [`STREET_NUTZARTEN`], alone or with its refinement (`Platz / Parkplatz`).
fn is_street(object: &MapObject) -> bool {
    object.tags.iter().any(|(k, v)| {
        k == "freemapper:nutzart"
            && STREET_NUTZARTEN.iter().any(|name| {
                v == name
                    || v.strip_prefix(name)
                        .is_some_and(|rest| rest.starts_with(" / "))
            })
    })
}

/// The minimum area (ground m²) of the 501.11 symbol `id`: the one the symbol set gives (the
/// bundled ISSprOM 2019-2 set: 0.5 mm²), else the open land minimum of the map kind, as the
/// training level uses it (ISSprOM 2019-2 401: 0.5 mm²; ISOM 2017-2 401: 0.55 mm x 0.55 mm).
/// Tolerant by [`TOLERANCE_MM`] on the side of the smallest square, as `drop_too_small` is.
fn min_area(id: u32, kind: MapKind, georef: &Georeferencing, symbol_set: &SymbolSet) -> f64 {
    let scale_denominator = georef.scale_denominator;
    let scale = SymbolScale::drawn_with(symbol_set, scale_denominator);
    let min_m2 = match symbol_set.minimum_area(id) {
        Some(area) => area.square_metres(scale_denominator),
        None if kind == MapKind::Forest => OPEN_LAND_MIN_FOREST.square_metres(scale),
        None => OPEN_LAND_MIN_SPRINT.square_metres(scale),
    };
    let side = (min_m2.sqrt() - PaperMm(TOLERANCE_MM).metres(scale_denominator)).max(0.0);
    side * side
}

/// The pieces of `carriageway` inside the union of the `streets` near it (map units).
fn inside_streets(
    carriageway: &Polygon<f64>,
    streets: &[(Rect<f64>, Polygon<f64>)],
) -> Vec<Polygon<f64>> {
    let Some(frame) = carriageway.bounding_rect() else {
        return Vec::new();
    };
    let near: Vec<&Polygon<f64>> = streets
        .iter()
        .filter(|(bounds, _)| bounds.intersects(&frame))
        .map(|(_, p)| p)
        .collect();
    if near.is_empty() {
        return Vec::new();
    }
    // Street areas can overlap a little where parcels do; the union keeps them once.
    MultiPolygon(vec![carriageway.clone()])
        .intersection(&geo::unary_union(near))
        .0
}

/// Whether more than half of the line `object` lies inside the `inner` street areas (map
/// units).
fn mostly_inside(object: &MapObject, inner: &[(Rect<f64>, Polygon<f64>)]) -> bool {
    // The parts are in mm on paper, Béziers flattened.
    let lines: Vec<LineString<f64>> = parts(&object.coords)
        .iter()
        .map(|p| {
            p.points
                .iter()
                .map(|&(x, y)| (PaperMm(x).map_units(), PaperMm(y).map_units()))
                .collect()
        })
        .collect();
    let total: f64 = lines.iter().map(line_length).sum();
    if total <= 0.0 {
        return false;
    }
    let lines = MultiLineString(lines);
    let Some(frame) = lines.bounding_rect() else {
        return false;
    };
    let near = MultiPolygon(
        inner
            .iter()
            .filter(|(bounds, _)| bounds.intersects(&frame))
            .map(|(_, p)| p.clone())
            .collect(),
    );
    if near.0.is_empty() {
        return false;
    }
    let inside: f64 = near.clip(&lines, false).0.iter().map(line_length).sum();
    2.0 * inside > total
}

/// The length of `line`, with `sqrt` only.
fn line_length(line: &LineString<f64>) -> f64 {
    line.0
        .windows(2)
        .map(|w| {
            let (dx, dy) = (w[1].x - w[0].x, w[1].y - w[0].y);
            (dx * dx + dy * dy).sqrt()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::super::tests::{area, georef_at, line, of_code};
    use super::super::{TrainingReport, train};
    use super::*;

    const SPRINT: u32 = 4000;

    fn sprint() -> SymbolSet {
        MapKind::Sprint.symbol_set(SPRINT)
    }

    fn run(set: &SymbolSet, objects: Vec<MapObject>) -> (Vec<MapObject>, TrainingReport) {
        train(objects, None, MapKind::Sprint, &georef_at(SPRINT), set)
    }

    fn tagged(mut object: MapObject, tags: &[(&str, &str)]) -> MapObject {
        object
            .tags
            .extend(tags.iter().map(|&(k, v)| (k.to_string(), v.to_string())));
        object
    }

    /// A street area (501.5 of a `Straßenverkehr` parcel) from `min` to `max` (paper mm).
    fn street(set: &SymbolSet, min: (f64, f64), max: (f64, f64)) -> MapObject {
        tagged(
            area(set, "501.5", min, max),
            &[
                ("freemapper:source", "alkis-nutzung"),
                ("freemapper:nutzart", "Straßenverkehr"),
            ],
        )
    }

    /// A hidden carriageway candidate from `min` to `max`.
    fn carriageway(set: &SymbolSet, min: (f64, f64), max: (f64, f64)) -> MapObject {
        tagged(
            area(set, "501.11", min, max),
            &[
                ("freemapper:source", "geoportal-berlin/sb-flaechen"),
                ("berlin:layer", "cm_fahrbahn"),
                (HIDDEN_KEY, HIDDEN_CARRIAGEWAY),
            ],
        )
    }

    fn kerb(set: &SymbolSet, points: &[(f64, f64)]) -> MapObject {
        tagged(
            line(set, "501.1", points),
            &[("freemapper:source", KERB_SOURCE)],
        )
    }

    fn area_mm2(object: &MapObject) -> f64 {
        object_polygon(object).unwrap().unsigned_area() / 1e6
    }

    #[test]
    fn the_training_map_draws_the_carriageway_darker() {
        let set = sprint();
        // A street 40 × 6 mm, a park beside it, the carriageway reaching 5 mm past the street's
        // ends; a second carriageway of 0.3 × 0.3 mm inside it is below the minimum.
        let park = tagged(
            area(&set, "501.5", (40.0, 0.0), (60.0, 6.0)),
            &[(
                "freemapper:nutzart",
                "Sport-, Freizeit- und Erholungsfläche",
            )],
        );
        let objects = vec![
            street(&set, (0.0, 0.0), (40.0, 6.0)),
            park,
            carriageway(&set, (-5.0, 1.5), (45.0, 4.5)),
            carriageway(&set, (10.0, 0.2), (10.3, 0.5)),
        ];
        let (out, report) = run(&set, objects);
        let heavy = of_code(&set, &out, "501.11");
        assert_eq!(heavy.len(), 1, "{heavy:?}");
        assert_eq!(report.carriageways, 1);
        let drawn = heavy[0];
        assert!(drawn.tags.iter().all(|(k, _)| k != HIDDEN_KEY), "{drawn:?}");
        assert!(
            drawn
                .tags
                .iter()
                .any(|(k, v)| k == "berlin:layer" && v == "cm_fahrbahn")
        );
        // Cut to the street: 40 × 3 mm, not into the park.
        let area = area_mm2(drawn);
        assert!((area - 120.0).abs() < 0.1, "{area}");
        let frame = object_polygon(drawn).unwrap().bounding_rect().unwrap();
        assert!(
            frame.min().x >= -2.0 && frame.max().x <= 40_002.0,
            "{frame:?}"
        );
        // The street areas stay as they were.
        assert_eq!(of_code(&set, &out, "501.5").len(), 2);
        assert!(
            !out.iter()
                .any(|o| o.tags.iter().any(|(k, _)| k == HIDDEN_KEY))
        );
    }

    #[test]
    fn kerb_lines_inside_a_street_go_and_the_border_stays() {
        let set = sprint();
        let border = tagged(
            line(&set, "501.1", &[(0.0, 0.0), (40.0, 0.0)]),
            &[
                ("freemapper:source", "generalization"),
                (
                    "freemapper:generalized",
                    "border of a paved area (ISSprOM 501)",
                ),
            ],
        );
        let objects = vec![
            street(&set, (0.0, 0.0), (40.0, 6.0)),
            carriageway(&set, (0.0, 1.5), (40.0, 4.5)),
            // Along the carriageway, inside the street: they go.
            kerb(&set, &[(2.0, 1.5), (38.0, 1.5)]),
            kerb(&set, &[(2.0, 4.5), (38.0, 4.5)]),
            // On the street's edge (0.1 mm = 0.4 m inside it): it stays, as the border does.
            kerb(&set, &[(2.0, 5.9), (38.0, 5.9)]),
            // A third inside, the rest beyond the street's end: it stays.
            kerb(&set, &[(30.0, 3.0), (60.0, 3.0)]),
            border,
        ];
        let (out, report) = run(&set, objects);
        assert_eq!(report.kerbs, 2);
        let edges = of_code(&set, &out, "501.1");
        let source = |o: &MapObject| {
            o.tags
                .iter()
                .find(|(k, _)| k == "freemapper:source")
                .map(|(_, v)| v.clone())
        };
        assert_eq!(edges.len(), 3, "{edges:?}");
        assert_eq!(
            edges
                .iter()
                .filter(|o| source(o).as_deref() == Some("generalization"))
                .count(),
            1
        );
        assert_eq!(of_code(&set, &out, "501.11").len(), 1);
    }

    #[test]
    fn without_a_survey_nothing_changes() {
        let set = sprint();
        let objects = vec![
            street(&set, (0.0, 0.0), (40.0, 6.0)),
            // A 501.1 of another source inside the street stays.
            tagged(
                line(&set, "501.1", &[(2.0, 1.5), (38.0, 1.5)]),
                &[("freemapper:source", "osm")],
            ),
        ];
        let mut trained = objects.clone();
        let report = draw_carriageways(&mut trained, MapKind::Sprint, &georef_at(SPRINT), &set);
        assert_eq!(report, Carriageways::default());
        assert_eq!(trained, objects);
        let (_, report) = run(&set, objects);
        assert_eq!(report.carriageways, 0);
        assert_eq!(report.kerbs, 0);
    }

    #[test]
    fn a_carriageway_without_a_street_goes() {
        let set = sprint();
        let mut objects = vec![
            area(&set, "501.5", (0.0, 0.0), (40.0, 6.0)),
            carriageway(&set, (0.0, 1.5), (40.0, 4.5)),
        ];
        let report = draw_carriageways(&mut objects, MapKind::Sprint, &georef_at(SPRINT), &set);
        assert_eq!(report.drawn, 0);
        assert_eq!(objects.len(), 1);
    }

    #[test]
    fn the_minimum_area_is_the_one_of_the_symbol_set() {
        let set = sprint();
        let georef = georef_at(SPRINT);
        let heavy = set.id_for_code("501.11").unwrap();
        // 0.5 mm² at 1:4000 are 8 m², a hair less for the tolerance.
        let min = min_area(heavy, MapKind::Sprint, &georef, &set);
        assert!(min < 8.0 && min > 7.9, "{min}");
    }
}
