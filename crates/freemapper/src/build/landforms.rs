// SPDX-License-Identifier: AGPL-3.0-or-later

//! Landform candidates from the landforms of `derive` (ADR 0104).

use geo::{Contains as _, Distance as _, InterpolateLine as _, Length as _, MapCoords as _};
use geo_types::{MultiPolygon, Polygon};

use crate::build::content::LANDFORM_KEY;
use crate::crs::ProjectedPoint;
use crate::derive::landforms::{Landform, LandformKind};
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::coord::MapCoord;
use crate::omap::{MapObject, MapObjectKind, SymbolSet};
use crate::sources::alkis::LandUse;

/// ISSprOM symbols of the landform candidates (`derive/landforms.rs`).
const LANDFORM_CODES: [(LandformKind, &str); 5] = [
    (LandformKind::Knoll, "109"),
    (LandformKind::ElongatedKnoll, "110"),
    (LandformKind::Depression, "111"),
    (LandformKind::Bank, "104"),
    (LandformKind::Cliff, "201"),
];
/// A knoll on bare rock is a rock tower or a block, not a knoll of the ground: a boulder (204) from 1 m
/// on, a large boulder (205) from 2 m on (ISOM 2017-2). Bare rock is the symbol 214 of the
/// objects drawn so far, which OpenStreetMap gives as `natural=bare_rock` (ADR 0199).
const BARE_ROCK_CODE: &str = "214";
const BOULDER_CODE: &str = "204";
const LARGE_BOULDER_CODE: &str = "205";
/// Boulders closer than their symbols allow (0.4 mm wide and 0.15 mm between symbols, centre to
/// centre) cannot be drawn singly: three or more in a chain are a boulder cluster (207).
const BOULDER_CLUSTER_CODE: &str = "207";
const BOULDER_GAP_MM: f64 = 0.55;
const BOULDER_CLUSTER_MIN: usize = 3;
const BOULDER_MIN_HEIGHT_M: f64 = 1.0;
const LARGE_BOULDER_MIN_HEIGHT_M: f64 = 2.0;
/// Lines already drawn that a bank or cliff from the terrain would only repeat: earth banks and
/// walls, cliffs, walls and fences.
const DRAWN_JUMP_CODES: [&str; 9] = [
    "104", "104.1", "105", "201", "513.1", "513.2", "515", "516", "518",
];
/// A cliff along roads and buildings is offered as an impassable wall: it is mostly a retaining
/// wall, and an earth bank would show something impassable as passable.
const RETAINING_WALL_CODE: &str = "515";
/// A landform this close to a building, or a bank this close to the water or to a drawn line,
/// is left out (ground metres): DGM1 is interpolated under buildings, and a wall or shore
/// already shows the jump.
const LANDFORM_CLEARANCE_M: f64 = 2.0;
/// A depression this close to a building is a light well, a basement stair or a hollow DGM1
/// makes between houses, not a hollow of the ground (ground metres).
const DEPRESSION_CLEARANCE_M: f64 = 8.0;
/// ALKIS land use where a knoll or depression is a heap, a garden mound, a courtyard or a ramp
/// rather than a form of the ground: built land, allotments and traffic areas. Schools and other
/// land of `Fläche besonderer funktionaler Prägung` stay open, because schoolyard maps lie there.
const BUILT_LAND_USE: [&str; 3] = [
    "Wohnbaufläche",
    "Fläche gemischter Nutzung",
    "Industrie- und Gewerbefläche",
];
const TRAFFIC_LAND_USE: [&str; 6] = [
    "Straßenverkehr",
    "Weg",
    "Platz",
    "Bahnverkehr",
    "Flugverkehr",
    "Schiffsverkehr",
];
const ALLOTMENT_LAND_USE: &str = "Kleingarten";
/// A cliff that runs mostly this close to a traffic area, or to a building, is a retaining wall
/// more often than a cliff: it is offered as [`RETAINING_WALL_CODE`] (ground metres).
const CLIFF_TRAFFIC_M: f64 = 3.0;
const CLIFF_BUILDING_M: f64 = 5.0;

/// The landforms of `derive`, with the terrain model they come from: its source and the newest
/// currency of its tiles, as tag values.
pub struct Landforms {
    pub forms: Vec<Landform>,
    pub source: String,
    pub currency: Option<String>,
}

/// Candidate objects for `landforms` (knolls 109, 110, depressions 111, banks 104, cliffs 201,
/// retaining walls 515) within `buffer_m` of `map_area`, each tagged [`LANDFORM_KEY`] so the
/// content settings place them as the group `landforms`. `drawn` are the objects of the map and
/// of the candidates map so far.
#[allow(clippy::too_many_arguments)]
pub fn landform_objects(
    landforms: &Landforms,
    footprints: &[Polygon<f64>],
    land_use: &[LandUse],
    water: &MultiPolygon<f64>,
    drawn: &[&MapObject],
    map_area: &Polygon<f64>,
    buffer_m: f64,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, GeorefError> {
    let Landforms {
        forms,
        source,
        currency,
    } = landforms;
    let jump_ids: Vec<u32> = DRAWN_JUMP_CODES
        .iter()
        .filter_map(|c| symbol_set.id_for_code(c))
        .collect();
    let drawn_lines: Vec<geo_types::LineString<f64>> = drawn
        .iter()
        .filter(|o| jump_ids.contains(&o.symbol_id) && matches!(o.kind, MapObjectKind::Path))
        .map(|o| {
            o.coords
                .iter()
                .map(|c| {
                    let p = georef.map_to_projected(*c);
                    (p.easting, p.northing)
                })
                .collect::<Vec<_>>()
                .into()
        })
        .collect();
    // The bare rock drawn so far, in projected metres.
    let rock: Vec<Polygon<f64>> = symbol_set
        .id_for_code(BARE_ROCK_CODE)
        .map(|id| {
            drawn
                .iter()
                .filter(|o| o.symbol_id == id)
                .filter_map(|o| crate::build::geometry::object_polygon(o))
                .map(|polygon| {
                    polygon.map_coords(|c| {
                        let p = georef.map_to_projected(MapCoord::new(c.x as i32, c.y as i32));
                        geo_types::Coord {
                            x: p.easting,
                            y: p.northing,
                        }
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let near_building = |p: &geo_types::Point<f64>| {
        footprints
            .iter()
            .any(|f| geo::Euclidean.distance(p, f) <= LANDFORM_CLEARANCE_M)
    };
    let land = |keep: &dyn Fn(&LandUse) -> bool| -> Vec<&geo_types::MultiPolygon<f64>> {
        land_use
            .iter()
            .filter(|l| keep(l))
            .map(|l| &l.footprint)
            .collect()
    };
    let built = land(&|l| {
        BUILT_LAND_USE.contains(&l.nutzart.as_str())
            || TRAFFIC_LAND_USE.contains(&l.nutzart.as_str())
            || l.bez.as_deref() == Some(ALLOTMENT_LAND_USE)
    });
    let traffic = land(&|l| TRAFFIC_LAND_USE.contains(&l.nutzart.as_str()));
    // Whether most of `line`, sampled every metre, is `near`.
    let mostly = |line: &geo_types::LineString<f64>,
                  near: &dyn Fn(&geo_types::Point<f64>) -> bool| {
        let samples = (geo::Euclidean.length(line) / 1.0).ceil().max(1.0) as usize;
        let close = (0..=samples)
            .filter_map(|k| {
                geo::Euclidean.point_at_ratio_from_start(line, k as f64 / samples as f64)
            })
            .filter(|p| near(p))
            .count();
        2 * close > samples + 1
    };
    // A bank is a repeat when most of it lies along a drawn line or the water.
    let repeats = |line: &geo_types::LineString<f64>| {
        mostly(line, &|p| {
            geo::Euclidean.distance(p, water) <= LANDFORM_CLEARANCE_M
                || drawn_lines
                    .iter()
                    .any(|l| geo::Euclidean.distance(p, l) <= LANDFORM_CLEARANCE_M)
        })
    };
    let along_roads = |line: &geo_types::LineString<f64>| {
        mostly(line, &|p| {
            traffic
                .iter()
                .any(|t| geo::Euclidean.distance(p, *t) <= CLIFF_TRAFFIC_M)
                || footprints
                    .iter()
                    .any(|f| geo::Euclidean.distance(p, f) <= CLIFF_BUILDING_M)
        })
    };
    let mut objects = Vec::new();
    // The boulders of bare rock, as (point, height, object): they may become clusters.
    let mut boulders: Vec<((f64, f64), f64, MapObject)> = Vec::new();
    for form in forms {
        let landform = form.kind;
        let Some(mut code) = LANDFORM_CODES
            .iter()
            .find(|(k, _)| *k == landform)
            .map(|(_, code)| *code)
        else {
            continue;
        };
        if landform == LandformKind::Cliff
            && along_roads(&geo_types::LineString::from(form.points.clone()))
        {
            code = RETAINING_WALL_CODE;
        }
        // A knoll on bare rock is a block or a tower of rock.
        if matches!(landform, LandformKind::Knoll | LandformKind::ElongatedKnoll)
            && form.height_m >= BOULDER_MIN_HEIGHT_M
            && rock
                .iter()
                .any(|r| r.contains(&geo_types::Point::from(form.points[0])))
        {
            let boulder = if form.height_m >= LARGE_BOULDER_MIN_HEIGHT_M {
                LARGE_BOULDER_CODE
            } else {
                BOULDER_CODE
            };
            if symbol_set.id_for_code(boulder).is_some() {
                code = boulder;
            }
        }
        let Some(symbol_id) = symbol_set.id_for_code(code) else {
            continue;
        };
        let geometry: geo_types::Geometry<f64> = if form.kind.is_point() {
            geo_types::Point::from(form.points[0]).into()
        } else {
            geo_types::LineString::from(form.points.clone()).into()
        };
        if geo::Euclidean.distance(&geometry, map_area) > buffer_m {
            continue;
        }
        match &geometry {
            geo_types::Geometry::Point(p) if near_building(p) => continue,
            // A line goes only when most of it is at a building: a terrace bank may end there.
            geo_types::Geometry::LineString(l) if mostly(l, &near_building) => continue,
            geo_types::Geometry::Point(p) if geo::Euclidean.distance(p, water) == 0.0 => continue,
            geo_types::Geometry::Point(p) if built.iter().any(|b| b.contains(p)) => continue,
            geo_types::Geometry::Point(p)
                if landform == LandformKind::Depression
                    && footprints
                        .iter()
                        .any(|f| geo::Euclidean.distance(p, f) <= DEPRESSION_CLEARANCE_M) =>
            {
                continue;
            }
            geo_types::Geometry::LineString(l) if repeats(l) => continue,
            _ => {}
        }
        let coords: Vec<MapCoord> = form
            .points
            .iter()
            .map(|(e, n)| {
                georef
                    .projected_to_map(ProjectedPoint {
                        easting: *e,
                        northing: *n,
                    })
                    .map(|c| MapCoord::new(c.x, c.y))
            })
            .collect::<Result<_, _>>()?;
        let kind = if form.kind.is_point() {
            // The rotation on paper: the direction of the knoll, turned by the grivation.
            let (e, n) = form.points[0];
            let r = form.rotation_deg.to_radians();
            let ahead = georef.projected_to_map(ProjectedPoint {
                easting: e + r.cos(),
                northing: n + r.sin(),
            })?;
            // Symbol 110 is long along its own y axis, so it turns a quarter less than its axis.
            let rotation_rad = if form.kind == LandformKind::ElongatedKnoll && code == "110" {
                f64::from(coords[0].y - ahead.y).atan2(f64::from(ahead.x - coords[0].x))
                    - std::f64::consts::FRAC_PI_2
            } else {
                0.0
            };
            MapObjectKind::Point { rotation_rad }
        } else {
            MapObjectKind::Path
        };
        let mut tags = vec![
            ("freemapper:source".to_string(), source.clone()),
            (
                "freemapper:height".to_string(),
                format!("{:.1}", form.height_m),
            ),
            (LANDFORM_KEY.to_string(), form.kind.name().to_string()),
        ];
        if let Some(currency) = currency {
            tags.push(("freemapper:currency".to_string(), currency.clone()));
        }
        let object = MapObject {
            symbol_id,
            kind,
            coords,
            tags,
        };
        if code == BOULDER_CODE || code == LARGE_BOULDER_CODE {
            boulders.push((form.points[0], form.height_m, object));
        } else {
            objects.push(object);
        }
    }
    objects.extend(boulders_and_clusters(
        boulders,
        georef,
        symbol_set,
        source,
        currency.as_deref(),
    )?);
    Ok(objects)
}

/// The boulders as they are, except that boulders closer together than their symbols allow
/// ([`BOULDER_GAP_MM`] on paper), three or more in a chain, become one boulder cluster (207) at
/// their centre, as high as the highest of them. Without the symbol they stay single.
fn boulders_and_clusters(
    boulders: Vec<((f64, f64), f64, MapObject)>,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
    source: &str,
    currency: Option<&str>,
) -> Result<Vec<MapObject>, GeorefError> {
    let Some(cluster_id) = symbol_set.id_for_code(BOULDER_CLUSTER_CODE) else {
        return Ok(boulders.into_iter().map(|(_, _, o)| o).collect());
    };
    let gap_m = BOULDER_GAP_MM * georef.metres_per_map_mm();
    // The chains of boulders: each boulder joins the group of every one within `gap_m`.
    let mut group: Vec<usize> = (0..boulders.len()).collect();
    for i in 0..boulders.len() {
        for j in 0..i {
            let ((e1, n1), (e2, n2)) = (boulders[i].0, boulders[j].0);
            if (e1 - e2).hypot(n1 - n2) <= gap_m {
                let (from, to) = (group[i], group[j]);
                for g in group.iter_mut().filter(|g| **g == from) {
                    *g = to;
                }
            }
        }
    }
    let mut out = Vec::new();
    let mut done = vec![false; boulders.len()];
    for first in 0..boulders.len() {
        if done[first] {
            continue;
        }
        let members: Vec<usize> = (first..boulders.len())
            .filter(|&k| group[k] == group[first])
            .collect();
        for &k in &members {
            done[k] = true;
        }
        if members.len() < BOULDER_CLUSTER_MIN {
            out.extend(members.iter().map(|&k| boulders[k].2.clone()));
            continue;
        }
        let count = members.len() as f64;
        let (e, n) = members.iter().fold((0.0, 0.0), |(e, n), &k| {
            (e + boulders[k].0.0 / count, n + boulders[k].0.1 / count)
        });
        let height = members.iter().map(|&k| boulders[k].1).fold(0.0, f64::max);
        let at = georef.projected_to_map(ProjectedPoint {
            easting: e,
            northing: n,
        })?;
        let mut tags = vec![
            ("freemapper:source".to_string(), source.to_string()),
            ("freemapper:height".to_string(), format!("{height:.1}")),
            (
                LANDFORM_KEY.to_string(),
                LandformKind::Knoll.name().to_string(),
            ),
            ("freemapper:boulders".to_string(), members.len().to_string()),
        ];
        if let Some(currency) = currency {
            tags.push(("freemapper:currency".to_string(), currency.to_string()));
        }
        out.push(MapObject {
            symbol_id: cluster_id,
            kind: MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![MapCoord::new(at.x, at.y)],
            tags,
        });
    }
    Ok(out)
}
