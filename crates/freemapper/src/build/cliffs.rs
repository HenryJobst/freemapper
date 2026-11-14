//! Cliffs on the forest map (ISOM 2017-2, 201 and 206; ADR 0126).
//!
//! OpenStreetMap draws rock country cliff by cliff: at the Bastei 2144 lines, up to 127 on one
//! square centimetre of paper, often several along one face and one face in several ways. The
//! norm keeps 0.25 mm between impassable features, draws no cliff shorter than 0.6 mm and shows a
//! rock pillar in plan shape (206). After the content settings placed the objects, the cliffs on
//! the map are generalized:
//!
//! * Each cliff is measured on the terrain model across it: the largest drop over 5 m within 8 m
//!   of the line, less what the slope around it drops anyway. That says how high the rock stands
//!   above its slope and which side is lower; the cliff is turned so that its tags point there.
//! * Pieces of one face that meet end to end are joined into one cliff.
//! * A cliff less than [`MIN_DROP_M`] above its slope goes to the candidates.
//! * A closed cliff with the lower side outside is a rock pillar: up to the norm's minimum it is
//!   enlarged, up to [`PILLAR_MAX_AREA_MM2`] it becomes the area 206. With the lower side inside
//!   it is a hollow and stays a cliff.
//! * The other cliffs are smoothed and kept, the highest first. A cliff keeps the norm's gap to
//!   those kept before and to the pillars, measured by the sides that face each other (line or
//!   tags); where it comes closer it is cut, and the pieces of at least 0.6 mm stay.
//!
//! The cliffs that go are hidden in the training map ([`HIDDEN_KEY`]` = generalized`), so it shows
//! the map's choice, and name why in `freemapper:generalized`. Returned is where the contours
//! stop: below every cliff of at least [`MIN_DROP_M`], kept or not, as far as its tags reach, and
//! around the pillars.

use geo::Simplify as _;
use geo::{
    Area as _, BoundingRect as _, Buffer as _, Distance as _, Euclidean, Length as _, unary_union,
};
use geo_types::{Coord, LineString, MultiPolygon, Polygon, Rect};

use crate::build::content::LANDFORM_KEY;
use crate::build::points::HIDDEN_KEY;
use crate::build::symbol_measures::{CLIFF_MM, CLIFF_TICK_MM};
use crate::crs::ProjectedPoint;
use crate::derive::ndsm::bilinear;
use crate::georef::Georeferencing;
use crate::measure::{PaperMm, SymbolMm, SymbolMm2, SymbolScale};
use crate::omap::coord::FLAG_CLOSE;
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet, closed_ring};
use crate::project::MapKind;
use crate::raster::Raster;

/// The impassable cliff and the gigantic boulder or rock pillar, in plan shape.
const CLIFF_CODE: &str = "201";
/// The top line of an impassable cliff, without tags: the norm lets them go where space is short.
const TOP_LINE_CODE: &str = "201.3";
const PILLAR_CODE: &str = "206";
/// Ruling: a cliff that stands less above its slope is no impassable cliff at forest scale.
pub const MIN_DROP_M: f64 = 3.0;
/// The cross profile: heights every metre up to this far from the line; the drop is the largest
/// over [`JUMP_M`]; the slope around it is read between [`TREND_NEAR_M`] and [`TREND_FAR_M`] on
/// each side, outside the face. A line drawn up to about 2 m off the face still finds it.
const PROFILE_M: i32 = 8;
const JUMP_M: i32 = 5;
const TREND_NEAR_M: i32 = 2;
const TREND_FAR_M: i32 = 7;
/// Below this difference between the two sides the terrain does not decide which side is lower,
/// and the cliff keeps the way OSM drew it (the top on the left, ticks on the right).
const SIDE_DECIDES_M: f64 = 1.0;
/// ISOM 2017-2, 201: the gap to any other impassable feature and the minimum length.
const CLIFF_GAP_MM: SymbolMm = SymbolMm(0.25);
const CLIFF_MIN_LENGTH_MM: SymbolMm = SymbolMm(0.6);
/// Ruling: a piece of a cut cliff is kept from this length; at 0.6 mm it would carry one or two
/// tags and print as a stub (the tags stand 0.5 mm apart).
const PIECE_MIN_LENGTH_MM: SymbolMm = SymbolMm(1.0);
/// Ruling: a top line without tags is kept from this length; shorter ones read as dashes, and
/// without tags nothing tells them from a wall or a piece of path (tried on the Bastei: at 1 mm
/// 122 top lines, mostly short strokes).
const TOP_LINE_MIN_LENGTH_MM: SymbolMm = SymbolMm(2.0);
/// ISOM 2017-2, 206: the minimum width and area, and the gap between pillars (the contours keep
/// it too).
const PILLAR_GAP_MM: SymbolMm = SymbolMm(0.15);
const PILLAR_MIN_WIDTH_MM: SymbolMm = SymbolMm(0.25);
const PILLAR_MIN_AREA_MM2: SymbolMm2 = SymbolMm2(0.3);
/// Ruling: a closed cliff around more is a plateau or a massif, no pillar (about 47 m across at
/// 1:15 000).
const PILLAR_MAX_AREA_MM2: SymbolMm2 = SymbolMm2(10.0);
/// Pieces of one face are joined when the end of one lies this close to the start of the next
/// and the face turns by less than [`JOIN_MAX_TURN_DEG`] there.
const JOIN_M: f64 = 1.0;
const JOIN_MAX_TURN_DEG: f64 = 60.0;
/// The lines are smoothed: OSM traces every niche of a rock face, and a bend shorter than about
/// the distance of two tags (0.5 mm) reads as zigzag. Gaussian along the line with this sigma
/// (ruling, from a comparison of 0, 3, 5 and 8 m on the Bastei; 0.3 mm is 4.5 m at 1:10 000);
/// then simplified like the contours.
const SMOOTH_SIGMA_MM: SymbolMm = SymbolMm(0.3);
const SMOOTH_STEP_M: f64 = 1.0;
const SIMPLIFY_MM: PaperMm = PaperMm(0.1);
/// How often along a cliff the profile is read, and the gap tested (ground metres).
const PROFILE_STEP_M: f64 = 2.0;
const GAP_STEP_M: f64 = 1.0;
/// The value of [`HIDDEN_KEY`] on a cliff the generalization took off the map, and on a knoll
/// or depression of the terrain model that stands on the rock.
pub const GENERALIZED: &str = "generalized";
pub const ROCK: &str = "rock";
const GENERALIZED_KEY: &str = "freemapper:generalized";
/// On a cliff turned by its profile: `orient_ticks` leaves it as it is.
pub const TICKS_KEY: &str = "freemapper:ticks";
pub const TICKS_BY_PROFILE: &str = "profile";

/// What the terrain says about a cliff.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Profile {
    /// How much more the ground drops across the cliff than the slope around it (metres).
    excess_m: f64,
    /// `Some(true)` when the lower side is on the right of the line, `Some(false)` on the left,
    /// `None` when the terrain does not decide.
    lower_right: Option<bool>,
}

/// One cliff of the map, after the pieces of a face were joined: in the map's CRS, with the
/// lower side on the right.
struct Cliff {
    /// The indices in `map` of the OSM cliffs it is made of.
    members: Vec<usize>,
    /// The line as drawn, smoothed and simplified ...
    line: LineString<f64>,
    /// ... and only simplified: the plan shape of a pillar, which smoothing would shrink.
    outline: LineString<f64>,
    closed: bool,
    /// `None` without terrain: then every cliff counts as tall and the longest go first.
    profile: Option<Profile>,
    /// The length of the smoothed line, and of the line as OSM drew it (the minimum length is
    /// measured on that one: smoothing shortens a crooked face).
    length_m: f64,
    outline_length_m: f64,
    frame: Rect<f64>,
}

impl Cliff {
    fn tall(&self) -> bool {
        tall_profile(self.profile)
    }

    /// The order in which cliffs are kept (ruling): height counts twice, so a high face goes
    /// before a long low terrace; without terrain the longest go first.
    fn weight(&self) -> f64 {
        let height = self.profile.map_or(1.0, |p| p.excess_m.max(0.0));
        height * height * self.length_m
    }
}

/// A cliff kept on the map: its line, frame, and whether it has its tags (a top line has none).
struct Kept {
    line: LineString<f64>,
    frame: Rect<f64>,
    tags: bool,
}

/// How much room a cliff has at one place, against the cliffs and pillars kept before.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Room {
    /// Room for its line and tags.
    Free,
    /// Room for its line only: there it is a top line without tags.
    LineOnly,
    /// Back to back with a kept cliff, closer than the gap: with it, one rock fin; the point
    /// of the kept cliff across.
    Fin(Coord<f64>),
    /// No room.
    Blocked,
}

/// The plan shape of a pillar and whether it was enlarged to the norm's minimum.
type Pillar = (Polygon<f64>, bool);

/// How far a cliff reaches from its centre line on each side, with the gap the norm keeps:
/// the line (half its width) on the left, the tags on the right.
struct Reach {
    line_m: f64,
    tags_m: f64,
    gap_m: f64,
}

/// Where the rock stands, as [`generalize_cliffs`] found it (map CRS).
#[derive(Debug, Default)]
pub struct Rock {
    /// Where the contours stop: below every cliff drawn, from its line to the end of its tags,
    /// and around the pillars.
    pub stops: Vec<Polygon<f64>>,
    /// Around every tall cliff and pillar, drawn or not, as far as the tags reach: the knolls and
    /// depressions of the terrain model there are the tops and clefts of the rock.
    pub mask: Vec<Polygon<f64>>,
}

/// Generalizes the impassable cliffs on the forest map (`map`); the ones that go are moved to
/// `candidates`. `terrain` is the height model of the project, in the map's CRS; without it every
/// cliff counts as tall and keeps the way OSM drew it. Returns where the rock stands.
pub fn generalize_cliffs(
    map: &mut Vec<MapObject>,
    candidates: &mut Vec<MapObject>,
    terrain: Option<&Raster<f32>>,
    map_kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Rock {
    let Some(cliff_id) = symbol_set.id_for_code(CLIFF_CODE) else {
        return Rock::default();
    };
    if map_kind != MapKind::Forest {
        return Rock::default();
    }
    let scale = SymbolScale::new(map_kind, georef.scale_denominator);
    let per_mm = scale.metres_per_symbol_mm();
    let reach = Reach {
        line_m: CLIFF_MM.0 / 2.0 * per_mm,
        tags_m: CLIFF_TICK_MM.0 * per_mm,
        gap_m: CLIFF_GAP_MM.0 * per_mm,
    };
    let tolerance = SIMPLIFY_MM.metres(georef.scale_denominator);
    let sigma_m = SMOOTH_SIGMA_MM.metres(scale);
    let min_length_m = CLIFF_MIN_LENGTH_MM.metres(scale);

    // The OSM cliffs, turned by their profile.
    let mut pieces: Vec<(usize, LineString<f64>, bool, Option<Profile>)> = Vec::new();
    for (index, o) in map.iter().enumerate() {
        if o.symbol_id != cliff_id
            || !matches!(o.kind, MapObjectKind::Path)
            || o.tags.iter().any(|(k, _)| k == HIDDEN_KEY)
            || o.coords.len() < 2
        {
            continue;
        }
        let closed = o.coords.len() > 2
            && (o.coords.first() == o.coords.last()
                || o.coords.iter().any(|c| c.flags & FLAG_CLOSE != 0));
        let mut points: Vec<Coord<f64>> = o
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
        if closed && points.first() != points.last() {
            points.push(points[0]);
        }
        let outline = LineString::new(points);
        let parts = match terrain {
            Some(t) => stretches(&outline, closed, t, min_length_m),
            None => vec![(outline, closed, None)],
        };
        for (mut line, closed, profile) in parts {
            if profile.is_some_and(|p| p.lower_right == Some(false)) {
                line.0.reverse();
            }
            pieces.push((index, line, closed, profile));
        }
    }
    if pieces.is_empty() {
        return Rock::default();
    }
    let cliffs: Vec<Cliff> = join_pieces(pieces)
        .into_iter()
        .filter_map(|(members, outline, closed, first)| {
            // A joined face is measured again as a whole; its side stays that of its pieces.
            let profile = match (members.len(), terrain) {
                (1, _) | (_, None) => first,
                (_, Some(t)) => profile(&outline, t).map(|p| Profile {
                    lower_right: Some(true),
                    ..p
                }),
            };
            let line = smooth(&outline, sigma_m, closed).simplify(tolerance);
            let outline = outline.simplify(tolerance);
            let frame = outline.bounding_rect()?;
            Some(Cliff {
                members,
                length_m: Euclidean.length(&line),
                outline_length_m: Euclidean.length(&outline),
                line,
                outline,
                closed,
                profile,
                frame,
            })
        })
        .collect();

    // The OSM cliffs the generalization replaced: by the cliffs, pieces and pillars it adds to the
    // map and the candidates.
    let mut replaced = vec![false; map.len()];
    let mut added_map: Vec<MapObject> = Vec::new();
    let mut added_candidates: Vec<MapObject> = Vec::new();
    let to_map = |line: &LineString<f64>, closed: bool| -> Option<Vec<MapCoord>> {
        let coords: Option<Vec<(i32, i32)>> = line
            .coords()
            .map(|c| {
                georef
                    .projected_to_map(ProjectedPoint {
                        easting: c.x,
                        northing: c.y,
                    })
                    .ok()
                    .map(|m| (m.x, m.y))
            })
            .collect();
        let coords = coords?;
        (coords.len() >= 2).then(|| {
            if closed && coords.len() > 3 {
                closed_ring(&coords)
            } else {
                coords
                    .into_iter()
                    .map(|(x, y)| MapCoord::new(x, y))
                    .collect()
            }
        })
    };
    // The object a cliff is written as: its first piece, with the ids of all.
    let object_of = |cliff: &Cliff, map: &[MapObject]| -> MapObject {
        let mut object = map[cliff.members[0]].clone();
        if let Some(coords) = to_map(&cliff.outline, cliff.closed) {
            object.coords = coords;
        }
        if cliff.members.len() > 1 {
            let ids: Vec<String> = cliff
                .members
                .iter()
                .filter_map(|&i| tag(&map[i], "freemapper:source_id"))
                .collect();
            if let Some((_, value)) = object
                .tags
                .iter_mut()
                .find(|(k, _)| k == "freemapper:source_id")
            {
                *value = ids.join("; ");
            }
            object.tags.push((
                GENERALIZED_KEY.to_string(),
                format!("joined from {} OSM cliffs", cliff.members.len()),
            ));
        }
        object
    };
    let hide = |mut object: MapObject, reason: String| -> MapObject {
        object
            .tags
            .push((HIDDEN_KEY.to_string(), GENERALIZED.to_string()));
        object.tags.push((GENERALIZED_KEY.to_string(), reason));
        object
    };

    // Low cliffs go. The tall ones are ranked together, walls and pillars (a closed cliff with the
    // lower side outside), the highest first: a pillar or a wall that comes too close to one kept
    // before gives way, a wall by being cut where it comes too close.
    let pillar_id = symbol_set.id_for_code(PILLAR_CODE);
    let min_area = PILLAR_MIN_AREA_MM2.square_metres(scale);
    let max_area = PILLAR_MAX_AREA_MM2.square_metres(scale);
    let half_width = PILLAR_MIN_WIDTH_MM.metres(scale) / 2.0;
    let pillar_gap_m = PILLAR_GAP_MM.metres(scale);
    let piece_min_m = PIECE_MIN_LENGTH_MM.metres(scale);
    let mut order: Vec<(&Cliff, Option<Pillar>)> = Vec::new();
    // Where a rock face stands, kept or not: the terrain's knolls there are its tops and clefts.
    let mut mask: Vec<Polygon<f64>> = Vec::new();
    for cliff in &cliffs {
        mark(&mut replaced, cliff);
        if !cliff.tall() {
            let reason = format!("cliff lower than {MIN_DROP_M} m above its slope");
            added_candidates.push(hide(object_of(cliff, map), reason));
            continue;
        }
        // Lower side on the right and the ring counter-clockwise: the rock is inside.
        let shape = Polygon::new(cliff.outline.clone(), Vec::new());
        let pillar = (pillar_id.is_some()
            && cliff.closed
            && shape.signed_area() > 0.0
            && shape.unsigned_area() <= max_area)
            .then(|| to_minimum(shape, min_area, half_width));
        match &pillar {
            Some((shape, _)) => mask.extend(shape.buffer(reach.tags_m).0),
            None => mask.extend(cliff.line.buffer(reach.tags_m).0),
        }
        order.push((cliff, pillar));
    }
    order.sort_by(|a, b| b.0.weight().total_cmp(&a.0.weight()));
    let mut kept: Vec<Kept> = Vec::new();
    let top_line_id = symbol_set.id_for_code(TOP_LINE_CODE);
    let mut kept_pillars: Vec<Polygon<f64>> = Vec::new();
    let mut stops: Vec<Polygon<f64>> = Vec::new();
    let ticks_by_profile = |cliff: &Cliff, object: &mut MapObject| {
        if cliff.profile.is_some_and(|p| p.lower_right.is_some()) {
            object
                .tags
                .push((TICKS_KEY.to_string(), TICKS_BY_PROFILE.to_string()));
        }
    };
    for (cliff, pillar) in order {
        let object = object_of(cliff, map);
        if let Some((shape, enlarged)) = pillar {
            let blocked = resample(shape.exterior(), GAP_STEP_M, true)
                .into_iter()
                .any(|(p, _)| kept.iter().any(|k| beside(p, k, &reach)))
                || kept_pillars
                    .iter()
                    .any(|other| Euclidean.distance(&shape, other) < pillar_gap_m);
            let coords = to_map(shape.exterior(), true);
            match (blocked, coords, pillar_id) {
                (false, Some(coords), Some(pillar_id)) => {
                    let mut object = object;
                    object.symbol_id = pillar_id;
                    object.coords = coords;
                    object.tags.push((
                        GENERALIZED_KEY.to_string(),
                        if enlarged {
                            "rock pillar from a closed cliff, enlarged to the norm's minimum"
                                .to_string()
                        } else {
                            "rock pillar from a closed cliff".to_string()
                        },
                    ));
                    added_map.push(object);
                    stops.extend(shape.buffer(pillar_gap_m).0);
                    kept_pillars.push(shape);
                }
                _ => {
                    let reason = format!(
                        "rock pillar closer than {} mm to a higher cliff or pillar kept before",
                        CLIFF_GAP_MM.0
                    );
                    added_candidates.push(hide(object, reason));
                }
            }
            continue;
        }
        if cliff.outline_length_m < min_length_m {
            let reason = format!("cliff shorter than {} mm", CLIFF_MIN_LENGTH_MM.0);
            added_candidates.push(hide(object, reason));
            continue;
        }
        let samples = resample(&cliff.line, GAP_STEP_M, cliff.closed);
        let rooms: Vec<Room> = samples
            .iter()
            .map(|&(p, d)| room(p, d, &kept, &kept_pillars, &reach))
            .collect();
        if rooms.iter().all(|r| matches!(r, Room::Free)) {
            let mut object = object;
            if let Some(coords) = to_map(&cliff.line, cliff.closed) {
                object.coords = coords;
            }
            ticks_by_profile(cliff, &mut object);
            added_map.push(object);
            stops.extend(below(&cliff.line, &reach));
            kept.push(Kept {
                line: cliff.line.clone(),
                frame: cliff.frame,
                tags: true,
            });
            continue;
        }
        let mut stays = 0;
        // Back to back with a kept cliff and closer than the gap: the two are one rock fin.
        for fin in fins(&samples, &rooms, min_length_m) {
            let (shape, _) = to_minimum(fin, 0.0, half_width);
            let (Some(pillar_id), Some(coords)) = (pillar_id, to_map(shape.exterior(), true))
            else {
                continue;
            };
            let mut body = object.clone();
            body.symbol_id = pillar_id;
            body.coords = coords;
            body.tags.push((
                GENERALIZED_KEY.to_string(),
                "rock fin between two cliffs back to back".to_string(),
            ));
            added_map.push(body);
            stops.extend(shape.buffer(pillar_gap_m).0);
            kept_pillars.push(shape);
            stays += 1;
        }
        // The rest keeps its tags where they have room and is a top line without tags where
        // only the line has.
        let top_line_min_m = TOP_LINE_MIN_LENGTH_MM.metres(scale);
        for (run, tags) in room_pieces(&samples, &rooms, piece_min_m, top_line_min_m, cliff.closed)
        {
            let run = run.simplify(tolerance);
            let symbol = if tags { Some(cliff_id) } else { top_line_id };
            let (Some(symbol), Some(coords), Some(frame)) =
                (symbol, to_map(&run, false), run.bounding_rect())
            else {
                continue;
            };
            let mut piece = object.clone();
            piece.symbol_id = symbol;
            piece.coords = coords;
            piece.tags.push((
                GENERALIZED_KEY.to_string(),
                if tags {
                    "piece of a cut cliff".to_string()
                } else {
                    "piece of a cut cliff, the top line only: its tags have no room".to_string()
                },
            ));
            if tags {
                ticks_by_profile(cliff, &mut piece);
            }
            added_map.push(piece);
            stops.extend(below(&run, &reach));
            kept.push(Kept {
                line: run,
                frame,
                tags,
            });
            stays += 1;
        }
        let reason = if stays == 0 {
            format!(
                "cliff closer than {} mm to a cliff or rock pillar kept before",
                CLIFF_GAP_MM.0
            )
        } else {
            format!(
                "cliff cut where it came closer than {} mm to a cliff or rock pillar kept \
                 before; {stays} piece(s) stay",
                CLIFF_GAP_MM.0
            )
        };
        added_candidates.push(hide(object, reason));
    }

    let mut kept_map = Vec::with_capacity(map.len());
    for (index, object) in map.drain(..).enumerate() {
        if !replaced[index] {
            kept_map.push(object);
        }
    }
    kept_map.extend(added_map);
    candidates.extend(added_candidates);
    *map = kept_map;
    Rock { stops, mask }
}

/// Hides the knolls and depressions of the terrain model (`freemapper:landform`) that stand in
/// `mask`: in rock country they are the tops of the towers and the clefts between them. One on the
/// map goes to the candidates; in the candidates it is hidden from the training map
/// ([`HIDDEN_KEY`]` = rock`). Returns how many.
pub fn hide_landforms_on_rock(
    map: &mut Vec<MapObject>,
    candidates: &mut Vec<MapObject>,
    mask: &[Polygon<f64>],
    georef: &Georeferencing,
) -> usize {
    use geo::Contains as _;
    if mask.is_empty() {
        return 0;
    }
    let frames: Vec<Option<Rect<f64>>> = mask.iter().map(|m| m.bounding_rect()).collect();
    let on_rock = |o: &MapObject| -> bool {
        if !matches!(o.kind, MapObjectKind::Point { .. })
            || !o.tags.iter().any(|(k, _)| k == LANDFORM_KEY)
            || o.tags.iter().any(|(k, _)| k == HIDDEN_KEY)
        {
            return false;
        }
        let Some(&c) = o.coords.first() else {
            return false;
        };
        let p = georef.map_to_projected(c);
        let point = geo_types::Point::new(p.easting, p.northing);
        mask.iter().zip(&frames).any(|(m, frame)| {
            frame.is_some_and(|f| {
                f.min().x <= p.easting
                    && p.easting <= f.max().x
                    && f.min().y <= p.northing
                    && p.northing <= f.max().y
            }) && m.contains(&point)
        })
    };
    let mut hidden = 0;
    let mark_rock = |o: &mut MapObject| {
        o.tags.push((HIDDEN_KEY.to_string(), ROCK.to_string()));
    };
    let (moved, stay): (Vec<MapObject>, Vec<MapObject>) = map.drain(..).partition(on_rock);
    *map = stay;
    for mut o in moved {
        mark_rock(&mut o);
        candidates.push(o);
        hidden += 1;
    }
    for o in candidates.iter_mut() {
        if on_rock(o) {
            mark_rock(o);
            hidden += 1;
        }
    }
    hidden
}

/// The value of `key` on `object`.
fn tag(object: &MapObject, key: &str) -> Option<String> {
    object
        .tags
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
}

/// Joins the open pieces whose end meets the start of another within [`JOIN_M`], turning by
/// less than [`JOIN_MAX_TURN_DEG`] (both run with the lower side on the right, so their tags
/// match). Each end joins at most one start, the closest; a chain whose last end meets its first
/// start is closed. Returns the members, the outline, whether closed, and the profile of the
/// first piece.
#[allow(clippy::type_complexity)]
fn join_pieces(
    pieces: Vec<(usize, LineString<f64>, bool, Option<Profile>)>,
) -> Vec<(Vec<usize>, LineString<f64>, bool, Option<Profile>)> {
    let n = pieces.len();
    let direction = |line: &LineString<f64>, at_end: bool| -> Option<(f64, f64)> {
        let c = &line.0;
        let (a, b) = if at_end {
            (c[c.len() - 2], c[c.len() - 1])
        } else {
            (c[0], c[1])
        };
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let l = dx.hypot(dy);
        (l > 1e-9).then(|| (dx / l, dy / l))
    };
    let min_cos = JOIN_MAX_TURN_DEG.to_radians().cos();
    // Candidate joins (gap, from, to), closest first.
    let mut joins: Vec<(f64, usize, usize)> = Vec::new();
    for (a, (_, la, ca, _)) in pieces.iter().enumerate() {
        if *ca || la.0.len() < 2 {
            continue;
        }
        let end = la.0[la.0.len() - 1];
        let tall_a = tall_profile(pieces[a].3);
        for (b, (_, lb, cb, pb)) in pieces.iter().enumerate() {
            // Only stretches of one class join: a tall face and the low ground beside it stay apart.
            if a == b || *cb || lb.0.len() < 2 || tall_profile(*pb) != tall_a {
                continue;
            }
            let start = lb.0[0];
            let gap = (end.x - start.x).hypot(end.y - start.y);
            if gap > JOIN_M {
                continue;
            }
            let (Some(da), Some(db)) = (direction(la, true), direction(lb, false)) else {
                continue;
            };
            if da.0 * db.0 + da.1 * db.1 >= min_cos {
                joins.push((gap, a, b));
            }
        }
    }
    joins.sort_by(|x, y| x.0.total_cmp(&y.0));
    let mut next: Vec<Option<usize>> = vec![None; n];
    let mut prev: Vec<Option<usize>> = vec![None; n];
    for (_, a, b) in joins {
        if next[a].is_none() && prev[b].is_none() {
            next[a] = Some(b);
            prev[b] = Some(a);
        }
    }
    // Chains start where no piece leads in; what is left are rings of pieces.
    let mut done = vec![false; n];
    let mut order: Vec<Vec<usize>> = Vec::new();
    for start in (0..n).filter(|&i| prev[i].is_none()).chain(0..n) {
        if done[start] {
            continue;
        }
        let mut members = Vec::new();
        let mut at = Some(start);
        while let Some(i) = at.filter(|&i| !done[i]) {
            done[i] = true;
            members.push(i);
            at = next[i];
        }
        order.push(members);
    }
    order
        .into_iter()
        .map(|members| {
            let first = &pieces[members[0]];
            if members.len() == 1 {
                return (vec![first.0], first.1.clone(), first.2, first.3);
            }
            let mut coords: Vec<Coord<f64>> = Vec::new();
            for &m in &members {
                let line = &pieces[m].1.0;
                let skip = usize::from(!coords.is_empty());
                coords.extend(line.iter().skip(skip).copied());
            }
            let last = *members.last().unwrap();
            let closed = next[last] == Some(members[0]);
            if closed {
                let start = coords[0];
                if let Some(end) = coords.last_mut() {
                    *end = start;
                }
            }
            let ids = members.iter().map(|&m| pieces[m].0).collect();
            (ids, LineString::new(coords), closed, first.3)
        })
        .collect()
}

/// Whether a profile counts as tall; without one every cliff does.
fn tall_profile(profile: Option<Profile>) -> bool {
    profile.is_none_or(|p| p.excess_m >= MIN_DROP_M)
}

/// Marks every OSM cliff `cliff` is made of as replaced.
fn mark(replaced: &mut [bool], cliff: &Cliff) {
    for &i in &cliff.members {
        replaced[i] = true;
    }
}

/// The cross profile of `line` on `terrain` every [`PROFILE_STEP_M`]: how far along the line,
/// and how much more the ground drops to the right and to the left than the slope beside it.
/// Samples where the terrain says nothing are left out.
fn profile_samples(line: &LineString<f64>, terrain: &Raster<f32>) -> Vec<(f64, f64, f64)> {
    let mut out = Vec::new();
    let mut along = 0.0;
    let mut last: Option<Coord<f64>> = None;
    for (p, d) in resample(line, PROFILE_STEP_M, false) {
        if let Some(q) = last {
            along += (p.x - q.x).hypot(p.y - q.y);
        }
        last = Some(p);
        // The right of the way (east, north) is (north, −east).
        let (nx, ny) = (d.1, -d.0);
        let heights: Option<Vec<f64>> = (-PROFILE_M..=PROFILE_M)
            .map(|t| {
                let t = f64::from(t);
                bilinear(terrain, p.x + nx * t, p.y + ny * t).map(f64::from)
            })
            .collect();
        let Some(heights) = heights else { continue };
        let at = |t: i32| heights[(t + PROFILE_M) as usize];
        let (mut down_right, mut down_left) = (f64::MIN, f64::MIN);
        for t in -PROFILE_M..=PROFILE_M - JUMP_M {
            let drop = at(t) - at(t + JUMP_M);
            down_right = down_right.max(drop);
            down_left = down_left.max(-drop);
        }
        // What the slope drops over the same distance to the right, beside the face.
        let trend =
            (at(-TREND_FAR_M) - at(-TREND_NEAR_M) + at(TREND_NEAR_M) - at(TREND_FAR_M)) / 2.0;
        out.push((
            along,
            down_right - trend.max(0.0),
            down_left - (-trend).max(0.0),
        ));
    }
    out
}

/// The [`Profile`] of profile samples: the medians of both sides.
fn aggregate(samples: &[(f64, f64, f64)]) -> Option<Profile> {
    let right = median(samples.iter().map(|s| s.1).collect())?;
    let left = median(samples.iter().map(|s| s.2).collect())?;
    let lower_right = ((right - left).abs() >= SIDE_DECIDES_M).then_some(right > left);
    Some(Profile {
        excess_m: right.max(left),
        lower_right,
    })
}

/// The profile of `line` on `terrain` (see [`Profile`]); `None` where the terrain says nothing.
fn profile(line: &LineString<f64>, terrain: &Raster<f32>) -> Option<Profile> {
    aggregate(&profile_samples(line, terrain))
}

/// `line` cut into stretches where its height or its lower side changes along it: a way that
/// runs from a face onto the slope above it, or round a tower and across the ridge behind. A
/// stretch shorter than `min_run_m` takes the class of its longer neighbour, so the class does
/// not flicker. Each stretch with its profile, whether closed (only a line of one class that was
/// closed stays closed).
#[allow(clippy::type_complexity)]
fn stretches(
    outline: &LineString<f64>,
    closed: bool,
    terrain: &Raster<f32>,
    min_run_m: f64,
) -> Vec<(LineString<f64>, bool, Option<Profile>)> {
    let samples = profile_samples(outline, terrain);
    if samples.is_empty() {
        return vec![(outline.clone(), closed, None)];
    }
    // The class of each sample: tall or not, and for a tall one the lower side, which an
    // undecided sample takes from the decided ones before or after it.
    let mut classes: Vec<(bool, Option<bool>)> = samples
        .iter()
        .map(|&(_, r, l)| {
            let tall = r.max(l) >= MIN_DROP_M;
            let side = ((r - l).abs() >= SIDE_DECIDES_M).then_some(r > l);
            (tall, if tall { side } else { None })
        })
        .collect();
    for pass in 0..2 {
        let mut last: Option<bool> = None;
        let indices: Vec<usize> = if pass == 0 {
            (0..classes.len()).collect()
        } else {
            (0..classes.len()).rev().collect()
        };
        for i in indices {
            if !classes[i].0 {
                last = None;
                continue;
            }
            match classes[i].1 {
                Some(side) => last = Some(side),
                None => classes[i].1 = last,
            }
        }
    }
    // Runs of one class: (first sample, last sample, class); then the short ones merge.
    let mut runs: Vec<(usize, usize, (bool, Option<bool>))> = Vec::new();
    for (i, class) in classes.iter().enumerate() {
        match runs.last_mut() {
            Some(run) if run.2 == *class => run.1 = i,
            _ => runs.push((i, i, *class)),
        }
    }
    let span = |run: &(usize, usize, (bool, Option<bool>))| {
        samples[run.1].0 - samples[run.0].0 + PROFILE_STEP_M
    };
    while runs.len() > 1 {
        let Some((k, _)) = runs
            .iter()
            .enumerate()
            .filter(|(_, r)| span(r) < min_run_m)
            .min_by(|a, b| span(a.1).total_cmp(&span(b.1)))
        else {
            break;
        };
        let into = match (k.checked_sub(1), runs.get(k + 1)) {
            (Some(before), Some(after)) => {
                if span(&runs[before]) >= span(after) {
                    before
                } else {
                    k + 1
                }
            }
            (Some(before), None) => before,
            (None, _) => k + 1,
        };
        let (lo, hi) = (k.min(into), k.max(into));
        let class = runs[into].2;
        runs[lo] = (runs[lo].0, runs[hi].1, class);
        runs.remove(hi);
        // Neighbours of one class join.
        let mut joined: Vec<(usize, usize, (bool, Option<bool>))> = Vec::new();
        for run in runs.drain(..) {
            match joined.last_mut() {
                Some(last) if last.2 == run.2 => last.1 = run.1,
                _ => joined.push(run),
            }
        }
        runs = joined;
    }
    let total = Euclidean.length(outline);
    let profile_of = |from: usize, to: usize, class: (bool, Option<bool>)| {
        aggregate(&samples[from..=to]).map(|p| Profile {
            lower_right: if class.0 { class.1 } else { p.lower_right },
            ..p
        })
    };
    if runs.len() == 1 {
        return vec![(
            outline.clone(),
            closed,
            profile_of(0, samples.len() - 1, runs[0].2),
        )];
    }
    // Cut halfway between the last sample of a run and the first of the next.
    let cut = |k: usize| (samples[runs[k].1].0 + samples[runs[k + 1].0].0) / 2.0;
    let mut out = Vec::new();
    let n = runs.len();
    let wraps = closed && runs[0].2 == runs[n - 1].2;
    for k in 0..n {
        let start = if k == 0 { 0.0 } else { cut(k - 1) };
        let end = if k == n - 1 { total } else { cut(k) };
        if wraps && k == n - 1 {
            continue;
        }
        let (line, from) = if wraps && k == 0 {
            let mut coords = substring(outline, cut(n - 2), total).0;
            coords.extend(substring(outline, 0.0, end).0.into_iter().skip(1));
            (LineString::new(coords), runs[n - 1].0)
        } else {
            (substring(outline, start, end), runs[k].0)
        };
        let profile = if wraps && k == 0 {
            let mut both: Vec<(f64, f64, f64)> = samples[from..].to_vec();
            both.extend_from_slice(&samples[..=runs[0].1]);
            aggregate(&both).map(|p| Profile {
                lower_right: if runs[0].2.0 {
                    runs[0].2.1
                } else {
                    p.lower_right
                },
                ..p
            })
        } else {
            profile_of(runs[k].0, runs[k].1, runs[k].2)
        };
        if line.0.len() >= 2 {
            out.push((line, false, profile));
        }
    }
    out
}

/// The part of `line` from `from` to `to` metres along it.
fn substring(line: &LineString<f64>, from: f64, to: f64) -> LineString<f64> {
    let mut out: Vec<Coord<f64>> = Vec::new();
    let mut along = 0.0;
    for segment in line.lines() {
        let length = segment.dx().hypot(segment.dy());
        let (a, b) = (along, along + length);
        along = b;
        if b < from || a > to || length < 1e-9 {
            continue;
        }
        let at = |d: f64| {
            let t = ((d - a) / length).clamp(0.0, 1.0);
            Coord {
                x: segment.start.x + segment.dx() * t,
                y: segment.start.y + segment.dy() * t,
            }
        };
        if out.is_empty() {
            out.push(at(from.max(a)));
        }
        out.push(at(to.min(b)));
    }
    out.dedup();
    LineString::new(out)
}

fn median(mut values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    Some(values[values.len() / 2])
}

/// `shape` grown to the norm's minimum area and width, and whether it had to grow.
fn to_minimum(shape: Polygon<f64>, min_area: f64, half_width: f64) -> (Polygon<f64>, bool) {
    use geo::Scale as _;
    let mut shape = shape;
    let mut enlarged = false;
    let area = shape.unsigned_area();
    if area > 0.0 && area < min_area {
        shape = shape.scale((min_area / area).sqrt());
        enlarged = true;
    }
    if shape.buffer(-half_width).unsigned_area() <= 0.0
        && let Some(wider) = shape
            .buffer(half_width)
            .0
            .into_iter()
            .max_by(|a, b| a.unsigned_area().total_cmp(&b.unsigned_area()))
    {
        shape = wider;
        enlarged = true;
    }
    (shape, enlarged)
}

/// Points every `step` metres along `line` with the unit direction of the line there; a closed
/// line leaves out its last point, which is its first.
fn resample(line: &LineString<f64>, step: f64, closed: bool) -> Vec<(Coord<f64>, (f64, f64))> {
    let mut out = Vec::new();
    for segment in line.lines() {
        let (dx, dy) = (segment.dx(), segment.dy());
        let length = dx.hypot(dy);
        if length < 1e-9 {
            continue;
        }
        let d = (dx / length, dy / length);
        let count = (length / step).ceil().max(1.0) as usize;
        for k in 0..count {
            let t = k as f64 / count as f64;
            out.push((
                Coord {
                    x: segment.start.x + dx * t,
                    y: segment.start.y + dy * t,
                },
                d,
            ));
        }
    }
    if !closed && let (Some(last), Some(&(_, d))) = (line.0.last(), out.last()) {
        out.push((*last, d));
    }
    out
}

/// The nearest point of `line` to `p` and the direction of `line` there.
fn nearest(line: &LineString<f64>, p: Coord<f64>) -> Option<(f64, Coord<f64>, (f64, f64))> {
    let mut best: Option<(f64, Coord<f64>, (f64, f64))> = None;
    for segment in line.lines() {
        let (dx, dy) = (segment.dx(), segment.dy());
        let length2 = dx * dx + dy * dy;
        if length2 < 1e-12 {
            continue;
        }
        let t = (((p.x - segment.start.x) * dx + (p.y - segment.start.y) * dy) / length2)
            .clamp(0.0, 1.0);
        let q = Coord {
            x: segment.start.x + dx * t,
            y: segment.start.y + dy * t,
        };
        let distance = (p.x - q.x).hypot(p.y - q.y);
        if best.is_none_or(|(b, _, _)| distance < b) {
            let length = length2.sqrt();
            best = Some((distance, q, (dx / length, dy / length)));
        }
    }
    best
}

/// Whether `to` lies on the right of the way `d` at `from`.
fn on_right(from: Coord<f64>, d: (f64, f64), to: Coord<f64>) -> bool {
    (to.x - from.x) * d.1 - (to.y - from.y) * d.0 > 0.0
}

/// How much room a cliff at `p`, running `d`, has against the `kept` cliffs and `pillars`: the
/// norm's gap between the sides that face each other, the line or the tags of each.
fn room(
    p: Coord<f64>,
    d: (f64, f64),
    kept: &[Kept],
    pillars: &[Polygon<f64>],
    reach: &Reach,
) -> Room {
    use geo::Contains as _;
    let rank = |r: &Room| match r {
        Room::Free => 0,
        Room::LineOnly => 1,
        Room::Fin(_) => 2,
        Room::Blocked => 3,
    };
    let mut worst = Room::Free;
    let most = reach.gap_m + 2.0 * reach.tags_m;
    for k in kept {
        let f = &k.frame;
        if p.x < f.min().x - most
            || p.x > f.max().x + most
            || p.y < f.min().y - most
            || p.y > f.max().y + most
        {
            continue;
        }
        let Some((distance, q, e)) = nearest(&k.line, p) else {
            continue;
        };
        let p_below_kept = on_right(q, e, p);
        let q_below_own = on_right(p, d, q);
        let kept_side = if k.tags && p_below_kept {
            reach.tags_m
        } else {
            reach.line_m
        };
        let own_side = if q_below_own {
            reach.tags_m
        } else {
            reach.line_m
        };
        if distance >= reach.gap_m + kept_side + own_side {
            continue;
        }
        let here = if !p_below_kept && !q_below_own {
            Room::Fin(q)
        } else if distance >= reach.gap_m + kept_side + reach.line_m {
            Room::LineOnly
        } else {
            Room::Blocked
        };
        if rank(&here) > rank(&worst) {
            worst = here;
        }
    }
    for pillar in pillars {
        if pillar.contains(&geo_types::Point::from(p)) {
            return Room::Blocked;
        }
        let Some((distance, q, _)) = nearest(pillar.exterior(), p) else {
            continue;
        };
        let own_side = if on_right(p, d, q) {
            reach.tags_m
        } else {
            reach.line_m
        };
        if distance >= reach.gap_m + own_side {
            continue;
        }
        let here = if distance >= reach.gap_m + reach.line_m {
            Room::LineOnly
        } else {
            Room::Blocked
        };
        if rank(&here) > rank(&worst) {
            worst = here;
        }
    }
    worst
}

/// The rock fins along a cliff: runs of [`Room::Fin`] samples at least `min_length_m` long, each
/// the strip between the cliff and the kept cliff across.
fn fins(
    samples: &[(Coord<f64>, (f64, f64))],
    rooms: &[Room],
    min_length_m: f64,
) -> Vec<Polygon<f64>> {
    let mut out = Vec::new();
    let mut run: Vec<(Coord<f64>, Coord<f64>)> = Vec::new();
    let mut close = |run: &mut Vec<(Coord<f64>, Coord<f64>)>| {
        let length: f64 = run
            .windows(2)
            .map(|w| (w[1].0.x - w[0].0.x).hypot(w[1].0.y - w[0].0.y))
            .sum();
        if run.len() >= 2 && length >= min_length_m {
            let mut ring: Vec<Coord<f64>> = run.iter().map(|(p, _)| *p).collect();
            ring.extend(run.iter().rev().map(|(_, q)| *q));
            ring.push(ring[0]);
            let strip = Polygon::new(LineString::new(ring), Vec::new());
            // A strip that crosses itself is cleaned by a buffer of nothing.
            if let Some(clean) = strip
                .buffer(0.0)
                .0
                .into_iter()
                .max_by(|a, b| a.unsigned_area().total_cmp(&b.unsigned_area()))
            {
                out.push(clean);
            }
        }
        run.clear();
    };
    for (&(p, _), room) in samples.iter().zip(rooms) {
        match room {
            Room::Fin(q) => run.push((p, *q)),
            _ => close(&mut run),
        }
    }
    close(&mut run);
    out
}

/// The pieces of a cliff that stay, with or without tags: runs of [`Room::Free`] samples keep
/// their tags, runs of free and [`Room::LineOnly`] samples are top lines; a run with tags shorter
/// than `min_m` loses them, and a top line shorter than `top_min_m` goes.
fn room_pieces(
    samples: &[(Coord<f64>, (f64, f64))],
    rooms: &[Room],
    min_m: f64,
    top_min_m: f64,
    closed: bool,
) -> Vec<(LineString<f64>, bool)> {
    let n = samples.len();
    // 2: tags, 1: line only, 0: nothing.
    let mut level: Vec<u8> = rooms
        .iter()
        .map(|r| match r {
            Room::Free => 2,
            Room::LineOnly => 1,
            Room::Fin(_) | Room::Blocked => 0,
        })
        .collect();
    // A closed line is read from a place without room, so that no run goes round its start.
    let start = if closed {
        level.iter().position(|&l| l == 0).unwrap_or(0)
    } else {
        0
    };
    let order: Vec<usize> = (0..n).map(|k| (start + k) % n).collect();
    let span = |from: usize, to: usize| -> f64 {
        (from..to)
            .map(|k| {
                let (a, b) = (samples[order[k]].0, samples[order[k + 1]].0);
                (b.x - a.x).hypot(b.y - a.y)
            })
            .sum()
    };
    // Runs with tags too short to carry them lose them.
    let mut k = 0;
    while k < n {
        let l = level[order[k]];
        let from = k;
        while k + 1 < n && level[order[k + 1]] == l {
            k += 1;
        }
        if l == 2 && span(from, k) < min_m {
            for i in from..=k {
                level[order[i]] = 1;
            }
        }
        k += 1;
    }
    // Runs of any room; inside them, the parts with tags and the parts without.
    let mut out = Vec::new();
    let mut k = 0;
    while k < n {
        if level[order[k]] == 0 {
            k += 1;
            continue;
        }
        let from = k;
        while k + 1 < n && level[order[k + 1]] != 0 {
            k += 1;
        }
        let to = k;
        k += 1;
        let mut part = from;
        while part <= to {
            let tags = level[order[part]] == 2;
            let mut end = part;
            while end < to && (level[order[end + 1]] == 2) == tags {
                end += 1;
            }
            // A piece reaches to the next sample, so the pieces of one run meet.
            let last = if end < to { end + 1 } else { end };
            if span(part, last) >= if tags { min_m } else { top_min_m } {
                let line: LineString<f64> = (part..=last).map(|i| samples[order[i]].0).collect();
                out.push((line, tags));
            }
            part = end + 1;
        }
    }
    out
}
/// Whether the edge of a pillar at `p` comes closer to the kept cliff `kept` than the norm's gap
/// from the side of the cliff that faces it.
fn beside(p: Coord<f64>, kept: &Kept, reach: &Reach) -> bool {
    let (frame, line) = (&kept.frame, &kept.line);
    let most = reach.gap_m + reach.tags_m;
    if p.x < frame.min().x - most
        || p.x > frame.max().x + most
        || p.y < frame.min().y - most
        || p.y > frame.max().y + most
    {
        return false;
    }
    let Some((distance, q, e)) = nearest(line, p) else {
        return false;
    };
    let side = if kept.tags && on_right(q, e, p) {
        reach.tags_m
    } else {
        reach.line_m
    };
    distance < reach.gap_m + side
}

/// Where a cliff stands on the ground with its symbol: from half its line on the left to the
/// end of its tags on the lower side.
fn below(line: &LineString<f64>, reach: &Reach) -> Vec<Polygon<f64>> {
    let quads: Vec<Polygon<f64>> = line
        .lines()
        .filter_map(|segment| {
            let (dx, dy) = (segment.dx(), segment.dy());
            let length = dx.hypot(dy);
            if length < 1e-9 {
                return None;
            }
            let (nx, ny) = (dy / length, -dx / length);
            let (a, b) = (segment.start, segment.end);
            let off = |c: Coord<f64>, t: f64| (c.x + nx * t, c.y + ny * t);
            Some(Polygon::new(
                LineString::from(vec![
                    off(a, -reach.line_m),
                    off(b, -reach.line_m),
                    off(b, reach.tags_m),
                    off(a, reach.tags_m),
                    off(a, -reach.line_m),
                ]),
                Vec::new(),
            ))
        })
        .collect();
    if quads.is_empty() {
        return Vec::new();
    }
    // The wedges at the bends are closed by a small buffer.
    let union: MultiPolygon<f64> = unary_union(&quads);
    union.buffer(reach.line_m).0
}

/// `line` resampled every [`SMOOTH_STEP_M`] and smoothed with a Gaussian of `sigma_m` along it.
/// An open line keeps its end points (the window shrinks towards them); a closed one wraps round.
fn smooth(line: &LineString<f64>, sigma_m: f64, closed: bool) -> LineString<f64> {
    let length = Euclidean.length(line);
    if sigma_m <= 0.0 || length < 2.0 * SMOOTH_STEP_M {
        return line.clone();
    }
    let count = (length / SMOOTH_STEP_M).round().max(2.0) as usize;
    let samples: Vec<Coord<f64>> = (0..=count)
        .filter_map(|k| {
            use geo::InterpolatableLine as _;
            line.point_at_ratio_from_start(&Euclidean, k as f64 / count as f64)
                .map(|p| p.0)
        })
        .collect();
    let n = samples.len();
    let sigma = sigma_m / (length / count as f64);
    let radius = (3.0 * sigma).ceil() as isize;
    let weights: Vec<f64> = (-radius..=radius)
        .map(|i| (-((i * i) as f64) / (2.0 * sigma * sigma)).exp())
        .collect();
    // A closed ring repeats its first point last; it wraps over the others.
    let ring = if closed { n - 1 } else { n };
    let smoothed: Vec<Coord<f64>> = (0..n)
        .map(|i| {
            if !closed && (i == 0 || i == n - 1) {
                return samples[i];
            }
            // Towards the ends of an open line the window shrinks to stay symmetric.
            let reach = if closed {
                radius
            } else {
                radius.min(i as isize).min((n - 1 - i) as isize)
            };
            let (mut x, mut y, mut w) = (0.0, 0.0, 0.0);
            for d in -reach..=reach {
                let j = (i as isize + d).rem_euclid(ring as isize) as usize;
                let weight = weights[(d + radius) as usize];
                x += samples[j].x * weight;
                y += samples[j].y * weight;
                w += weight;
            }
            Coord { x: x / w, y: y / w }
        })
        .collect();
    let mut smoothed = smoothed;
    if closed {
        smoothed[n - 1] = smoothed[0];
    }
    LineString::new(smoothed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::points::test_support::{Fixture, fixture};
    use crate::raster::GridSpec;
    use geo::Contains as _;
    use geo_types::Point;

    type P = (f64, f64);

    /// The point `at` (metres on the map from the reference point) in the map's CRS.
    fn projected(f: &Fixture, at: P) -> P {
        let p = f.georef.map_to_projected(f.at(at));
        (p.easting, p.northing)
    }

    /// Whether `point` (map CRS) lies in one of `rock`.
    fn in_rock(rock: &[Polygon<f64>], point: P) -> bool {
        rock.iter()
            .any(|r| r.contains(&Point::new(point.0, point.1)))
    }

    fn cliff(f: &Fixture, points: &[P]) -> MapObject {
        MapObject {
            symbol_id: f.id(CLIFF_CODE),
            kind: MapObjectKind::Path,
            coords: points.iter().map(|&p| f.at(p)).collect(),
            tags: vec![("freemapper:source".to_string(), "osm".to_string())],
        }
    }

    /// A terrain 400 m square around the reference point, 1 m cells, with `z(de, dn)` at the
    /// cell centres, in metres right of and above the reference point on the map (the map is
    /// turned against the grid, so the heights follow the map's axes like the test lines).
    fn terrain(f: &Fixture, z: impl Fn(f64, f64) -> f32) -> Raster<f32> {
        let (e0, n0) = (
            f.georef.ref_projected.easting,
            f.georef.ref_projected.northing,
        );
        let grid = GridSpec {
            west: e0 - 200.0,
            north: n0 + 200.0,
            pixel_size: 1.0,
            width: 400,
            height: 400,
        };
        let mut r = Raster::filled(grid, 0.0);
        for y in 0..400 {
            for x in 0..400 {
                let at = f
                    .georef
                    .projected_to_map(ProjectedPoint {
                        easting: e0 - 199.5 + f64::from(x),
                        northing: n0 + 199.5 - f64::from(y),
                    })
                    .unwrap();
                r.set(
                    x,
                    y,
                    z(f64::from(at.x) / f.metre(), -f64::from(at.y) / f.metre()),
                );
            }
        }
        r
    }

    /// Steps of `heights[i]` metres down, southwards, at the norths `steps[i]`.
    fn terraces(steps: &'static [(f64, f32)]) -> impl Fn(f64, f64) -> f32 {
        move |_, dn| {
            100.0
                + steps
                    .iter()
                    .filter(|(n, _)| dn > *n)
                    .map(|(_, h)| h)
                    .sum::<f32>()
        }
    }

    fn run(
        f: &Fixture,
        map: &mut Vec<MapObject>,
        terrain: Option<&Raster<f32>>,
    ) -> (Vec<MapObject>, Vec<Polygon<f64>>) {
        let mut candidates = Vec::new();
        let rock = generalize_cliffs(map, &mut candidates, terrain, f.kind, &f.georef, &f.set);
        (candidates, rock.stops)
    }

    fn generalized(o: &MapObject) -> Option<&str> {
        o.tags
            .iter()
            .find(|(k, _)| k == GENERALIZED_KEY)
            .map(|(_, v)| v.as_str())
    }

    fn hidden(o: &MapObject) -> bool {
        o.tags
            .iter()
            .any(|(k, v)| k == HIDDEN_KEY && v == GENERALIZED)
    }

    fn of_code<'a>(f: &Fixture, list: &'a [MapObject], code: &str) -> Vec<&'a MapObject> {
        list.iter().filter(|o| o.symbol_id == f.id(code)).collect()
    }

    /// The ground points of `object` in metres on the map from the reference point.
    fn at_metres(f: &Fixture, object: &MapObject) -> Vec<P> {
        object
            .coords
            .iter()
            .map(|c| (f64::from(c.x) / f.metre(), -f64::from(c.y) / f.metre()))
            .collect()
    }

    // At 1:15 000 with symbols at 100 %: 1 mm is 15 m. The line reaches 2.625 m, the tags
    // 8.625 m, the gap is 3.75 m: back to back 9 m, staggered 15 m, facing 21 m.

    #[test]
    fn a_low_cliff_goes_to_the_candidates_hidden() {
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, terraces(&[(0.0, 1.5)]));
        let mut map = vec![cliff(&f, &[(-50.0, 0.0), (50.0, 0.0)])];
        let (candidates, rock) = run(&f, &mut map, Some(&ground));
        assert!(map.is_empty());
        assert_eq!(candidates.len(), 1);
        assert!(hidden(&candidates[0]));
        assert_eq!(
            generalized(&candidates[0]),
            Some("cliff lower than 3 m above its slope")
        );
        assert!(rock.is_empty(), "a low cliff stops no contour");
    }

    #[test]
    fn a_steep_slope_without_rock_is_no_cliff() {
        // 40° all the way: 4.2 m over 5 m, but the slope beside it drops as much.
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, |_, dn| 100.0 + 0.84 * dn as f32);
        let mut map = vec![cliff(&f, &[(-50.0, 0.0), (50.0, 0.0)])];
        let (candidates, _) = run(&f, &mut map, Some(&ground));
        assert!(map.is_empty());
        assert_eq!(
            generalized(&candidates[0]),
            Some("cliff lower than 3 m above its slope")
        );
    }

    #[test]
    fn of_two_staggered_cliffs_too_close_the_higher_stays() {
        // Both drop to the south, 10 m apart: the tags of the upper one reach the lower one.
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, terraces(&[(0.0, 4.0), (10.0, 12.0)]));
        let mut map = vec![
            cliff(&f, &[(-60.0, 0.0), (60.0, 0.0)]),
            cliff(&f, &[(-60.0, 10.0), (60.0, 10.0)]),
        ];
        let (candidates, rock) = run(&f, &mut map, Some(&ground));
        assert_eq!(map.len(), 1);
        assert!(
            (at_metres(&f, &map[0])[0].1 - 10.0).abs() < 0.5,
            "the 12 m cliff stays"
        );
        assert_eq!(candidates.len(), 1);
        assert!(hidden(&candidates[0]));
        assert_eq!(
            generalized(&candidates[0]),
            Some("cliff closer than 0.25 mm to a cliff or rock pillar kept before")
        );
        // The contours stop below the cliff drawn only.
        assert!(!in_rock(&rock, projected(&f, (0.0, -4.0))));
        assert!(in_rock(&rock, projected(&f, (0.0, 6.0))));
    }

    #[test]
    fn staggered_cliffs_far_enough_apart_both_stay() {
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, terraces(&[(0.0, 4.0), (20.0, 12.0)]));
        let mut map = vec![
            cliff(&f, &[(-60.0, 0.0), (60.0, 0.0)]),
            cliff(&f, &[(-60.0, 20.0), (60.0, 20.0)]),
        ];
        let (candidates, _) = run(&f, &mut map, Some(&ground));
        assert_eq!(map.len(), 2);
        assert!(candidates.is_empty());
    }

    #[test]
    fn back_to_back_cliffs_need_less_room_facing_ones_more() {
        let f = fixture(MapKind::Forest);
        // A rock wall 10 m wide and 10 m high: its two faces 10 m apart, tags outwards.
        let ridge = terrain(
            &f,
            |_, dn| if dn > 0.0 && dn < 10.0 { 110.0 } else { 100.0 },
        );
        let mut map = vec![
            cliff(&f, &[(-60.0, 0.0), (60.0, 0.0)]),
            cliff(&f, &[(-60.0, 10.0), (60.0, 10.0)]),
        ];
        let (candidates, _) = run(&f, &mut map, Some(&ridge));
        assert_eq!(map.len(), 2, "back to back: 9 m are enough");
        assert!(candidates.is_empty());
        // The second is turned, so that its tags point down to the north.
        let north = at_metres(
            &f,
            map.iter().find(|o| at_metres(&f, o)[0].1 > 5.0).unwrap(),
        );
        assert!(north[0].0 > north[north.len() - 1].0, "{north:?}");
        // A gully 15 m wide: the tags face each other and need 21 m.
        let gully = terrain(&f, |_, dn| if dn > 0.0 && dn < 15.0 { 90.0 } else { 100.0 });
        let mut map = vec![
            cliff(&f, &[(-60.0, 0.0), (60.0, 0.0)]),
            cliff(&f, &[(-60.0, 15.0), (60.0, 15.0)]),
        ];
        let (candidates, _) = run(&f, &mut map, Some(&gully));
        assert_eq!(map.len(), 1);
        assert_eq!(candidates.len(), 1);
    }

    #[test]
    fn a_cliff_too_close_in_part_is_cut_and_its_far_piece_stays() {
        let f = fixture(MapKind::Forest);
        // A high face from -60 to 60 m, a lower one 10 m south of it from 0 to 150 m.
        let ground = terrain(&f, |de, dn| {
            let mut z = 100.0;
            if dn > 0.0 && (-60.0..=60.0).contains(&de) {
                z += 12.0;
            }
            if dn > -10.0 {
                z += 4.0;
            }
            z
        });
        let mut map = vec![
            cliff(&f, &[(-60.0, 0.0), (60.0, 0.0)]),
            cliff(&f, &[(0.0, -10.0), (150.0, -10.0)]),
        ];
        let (candidates, _) = run(&f, &mut map, Some(&ground));
        assert_eq!(map.len(), 2);
        let piece = map
            .iter()
            .find(|o| generalized(o) == Some("piece of a cut cliff"))
            .expect("a piece stays");
        let points = at_metres(&f, piece);
        let start = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        // Staggered, 10 m apart: 15 m between the lines, so the piece starts about 11 m past
        // the end of the high face.
        assert!((68.0..76.0).contains(&start), "{start}");
        assert_eq!(candidates.len(), 1);
        assert!(hidden(&candidates[0]));
        assert!(
            generalized(&candidates[0])
                .unwrap()
                .starts_with("cliff cut where")
        );
    }

    #[test]
    fn a_cliff_shorter_than_the_norm_goes() {
        // 0.6 mm at 1:15 000: 9 m.
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, terraces(&[(0.0, 6.0)]));
        let mut map = vec![cliff(&f, &[(0.0, 0.0), (7.0, 0.0)])];
        let (candidates, _) = run(&f, &mut map, Some(&ground));
        assert!(map.is_empty());
        assert_eq!(
            generalized(&candidates[0]),
            Some("cliff shorter than 0.6 mm")
        );
    }

    #[test]
    fn pieces_of_one_face_are_joined() {
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, terraces(&[(0.0, 6.0)]));
        let mut first = cliff(&f, &[(-60.0, 0.0), (0.0, 0.0)]);
        first
            .tags
            .push(("freemapper:source_id".into(), "way/1".into()));
        let mut second = cliff(&f, &[(0.5, 0.0), (60.0, 0.0)]);
        second
            .tags
            .push(("freemapper:source_id".into(), "way/2".into()));
        let mut map = vec![first, second];
        let (candidates, _) = run(&f, &mut map, Some(&ground));
        assert!(candidates.is_empty());
        assert_eq!(map.len(), 1);
        assert_eq!(generalized(&map[0]), Some("joined from 2 OSM cliffs"));
        assert_eq!(
            tag(&map[0], "freemapper:source_id").as_deref(),
            Some("way/1; way/2")
        );
        assert_eq!(tag(&map[0], TICKS_KEY).as_deref(), Some(TICKS_BY_PROFILE));
    }

    const SQUARE: [P; 5] = [
        (-10.0, -10.0),
        (10.0, -10.0),
        (10.0, 10.0),
        (-10.0, 10.0),
        (-10.0, -10.0),
    ];

    #[test]
    fn a_closed_cliff_round_a_pillar_becomes_206() {
        let f = fixture(MapKind::Forest);
        // A tower 20 m square and 15 m high.
        let ground = terrain(&f, |de, dn| {
            if de.abs() < 10.0 && dn.abs() < 10.0 {
                115.0
            } else {
                100.0
            }
        });
        let mut map = vec![cliff(&f, &SQUARE)];
        let (candidates, rock) = run(&f, &mut map, Some(&ground));
        assert!(candidates.is_empty(), "the pillar is the same rock");
        assert_eq!(map.len(), 1);
        assert_eq!(map[0].symbol_id, f.id(PILLAR_CODE));
        assert_eq!(
            generalized(&map[0]),
            Some("rock pillar from a closed cliff")
        );
        assert!(in_rock(&rock, projected(&f, (0.0, 0.0))));
    }

    #[test]
    fn a_small_pillar_is_enlarged_to_the_minimum() {
        // 6 m square: 36 m², the norm asks 0.3 mm², 67.5 m².
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, |de, dn| {
            if de.abs() < 3.0 && dn.abs() < 3.0 {
                112.0
            } else {
                100.0
            }
        });
        let ring: Vec<P> = SQUARE.iter().map(|(e, n)| (e * 0.3, n * 0.3)).collect();
        let mut map = vec![cliff(&f, &ring)];
        let (candidates, _) = run(&f, &mut map, Some(&ground));
        assert!(candidates.is_empty());
        assert_eq!(map.len(), 1);
        assert_eq!(map[0].symbol_id, f.id(PILLAR_CODE));
        assert_eq!(
            generalized(&map[0]),
            Some("rock pillar from a closed cliff, enlarged to the norm's minimum")
        );
        let area =
            Polygon::new(LineString::from(at_metres(&f, &map[0])), Vec::new()).unsigned_area();
        assert!(area >= 67.0, "{area}");
    }

    #[test]
    fn a_closed_cliff_round_a_hollow_stays_a_cliff() {
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, |de, dn| {
            if de.abs() < 10.0 && dn.abs() < 10.0 {
                90.0
            } else {
                100.0
            }
        });
        let mut map = vec![cliff(&f, &SQUARE)];
        let (candidates, _) = run(&f, &mut map, Some(&ground));
        assert!(candidates.is_empty());
        assert_eq!(of_code(&f, &map, CLIFF_CODE).len(), 1);
        assert!(of_code(&f, &map, PILLAR_CODE).is_empty());
    }

    #[test]
    fn a_cliff_beside_a_pillar_is_cut() {
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, |de, dn| {
            if de.abs() < 10.0 && dn.abs() < 10.0 {
                120.0
            } else if dn > 14.0 {
                110.0
            } else {
                100.0
            }
        });
        let mut map = vec![
            cliff(&f, &SQUARE),
            cliff(&f, &[(-80.0, 14.0), (80.0, 14.0)]),
        ];
        let (candidates, _) = run(&f, &mut map, Some(&ground));
        assert_eq!(of_code(&f, &map, PILLAR_CODE).len(), 1);
        // The face north of the pillar keeps its two ends, not its middle.
        assert_eq!(of_code(&f, &map, CLIFF_CODE).len(), 2);
        assert!(
            generalized(&candidates[0])
                .unwrap()
                .starts_with("cliff cut where")
        );
    }

    #[test]
    fn the_contours_stop_below_a_cliff_only() {
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, terraces(&[(0.0, 6.0)]));
        let mut map = vec![cliff(&f, &[(-60.0, 0.0), (60.0, 0.0)])];
        let (_, rock) = run(&f, &mut map, Some(&ground));
        assert!(
            in_rock(&rock, projected(&f, (0.0, -7.0))),
            "below, under the tags"
        );
        assert!(
            !in_rock(&rock, projected(&f, (0.0, 6.0))),
            "above, on the top"
        );
    }

    #[test]
    fn a_way_is_cut_where_its_height_changes() {
        // A face 6 m high west of the reference point, a step of 1 m east of it.
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, |de, dn| {
            let step = if de < 0.0 { 6.0 } else { 1.0 };
            if dn > 0.0 { 100.0 + step } else { 100.0 }
        });
        let mut map = vec![cliff(&f, &[(-60.0, 0.0), (60.0, 0.0)])];
        let (candidates, _) = run(&f, &mut map, Some(&ground));
        assert_eq!(map.len(), 1);
        let kept = at_metres(&f, &map[0]);
        let east = kept.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
        assert!(
            (-6.0..=6.0).contains(&east),
            "the tall stretch ends near 0: {east}"
        );
        assert_eq!(candidates.len(), 1);
        assert_eq!(
            generalized(&candidates[0]),
            Some("cliff lower than 3 m above its slope")
        );
        let low = at_metres(&f, &candidates[0]);
        assert!(low.iter().all(|p| p.0 > -6.0), "{low:?}");
    }

    #[test]
    fn a_higher_wall_beats_a_pillar_beside_it() {
        let f = fixture(MapKind::Forest);
        // A tower 15 m high, and 10 m north of it a face of 20 m whose tags point at it.
        let ground = terrain(&f, |de, dn| {
            if dn > 20.0 {
                120.0
            } else if de.abs() < 10.0 && dn.abs() < 10.0 {
                115.0
            } else {
                100.0
            }
        });
        let mut map = vec![
            cliff(&f, &SQUARE),
            cliff(&f, &[(-80.0, 20.0), (80.0, 20.0)]),
        ];
        let (candidates, _) = run(&f, &mut map, Some(&ground));
        assert!(of_code(&f, &map, PILLAR_CODE).is_empty());
        assert_eq!(of_code(&f, &map, CLIFF_CODE).len(), 1);
        assert_eq!(
            generalized(&candidates[0]),
            Some("rock pillar closer than 0.25 mm to a higher cliff or pillar kept before")
        );
    }

    #[test]
    fn knolls_on_the_rock_are_hidden() {
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, terraces(&[(0.0, 8.0)]));
        let mut map = vec![cliff(&f, &[(-60.0, 0.0), (60.0, 0.0)])];
        let mut candidates = Vec::new();
        let rock = generalize_cliffs(
            &mut map,
            &mut candidates,
            Some(&ground),
            f.kind,
            &f.georef,
            &f.set,
        );
        let knoll = |at: P| MapObject {
            symbol_id: f.id("109"),
            kind: MapObjectKind::Point { rotation_rad: 0.0 },
            coords: vec![f.at(at)],
            tags: vec![(LANDFORM_KEY.to_string(), "knoll".to_string())],
        };
        let mut landforms = vec![knoll((0.0, 5.0)), knoll((0.0, 40.0))];
        let hidden = hide_landforms_on_rock(&mut map, &mut landforms, &rock.mask, &f.georef);
        assert_eq!(hidden, 1);
        assert_eq!(tag(&landforms[0], HIDDEN_KEY).as_deref(), Some(ROCK));
        assert_eq!(tag(&landforms[1], HIDDEN_KEY), None);
    }

    #[test]
    fn a_cliff_whose_tags_have_no_room_is_a_top_line() {
        // The lower face is the higher (12 m) and is kept; the upper one (4 m), 10 m north,
        // would point its tags at it: with tags 15 m are needed, without 9 m.
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, terraces(&[(0.0, 12.0), (10.0, 4.0)]));
        let mut map = vec![
            cliff(&f, &[(-60.0, 0.0), (60.0, 0.0)]),
            cliff(&f, &[(-60.0, 10.0), (60.0, 10.0)]),
        ];
        let (candidates, _) = run(&f, &mut map, Some(&ground));
        assert_eq!(of_code(&f, &map, CLIFF_CODE).len(), 1);
        let top = of_code(&f, &map, TOP_LINE_CODE);
        assert_eq!(top.len(), 1);
        assert_eq!(
            generalized(top[0]),
            Some("piece of a cut cliff, the top line only: its tags have no room")
        );
        assert_eq!(
            candidates.len(),
            1,
            "the whole cliff, hidden, for the mapper"
        );
    }

    #[test]
    fn two_cliffs_back_to_back_closer_than_the_gap_are_a_rock_fin() {
        // A fin 6 m wide and 10 m high: back to back the faces need 9 m.
        let f = fixture(MapKind::Forest);
        let ground = terrain(&f, |_, dn| if dn > 0.0 && dn < 6.0 { 110.0 } else { 100.0 });
        let mut map = vec![
            cliff(&f, &[(-60.0, 0.0), (60.0, 0.0)]),
            cliff(&f, &[(-60.0, 6.0), (60.0, 6.0)]),
        ];
        let (candidates, rock) = run(&f, &mut map, Some(&ground));
        assert_eq!(of_code(&f, &map, CLIFF_CODE).len(), 1);
        let fin = of_code(&f, &map, PILLAR_CODE);
        assert_eq!(fin.len(), 1);
        assert_eq!(
            generalized(fin[0]),
            Some("rock fin between two cliffs back to back")
        );
        assert_eq!(candidates.len(), 1);
        assert!(
            in_rock(&rock, projected(&f, (0.0, 3.0))),
            "no contour in the fin"
        );
    }

    #[test]
    fn without_terrain_every_cliff_counts_as_tall() {
        let f = fixture(MapKind::Forest);
        let mut map = vec![cliff(&f, &[(-50.0, 0.0), (50.0, 0.0)])];
        let (candidates, rock) = run(&f, &mut map, None);
        assert_eq!(map.len(), 1);
        assert!(candidates.is_empty());
        assert!(!rock.is_empty());
        assert_eq!(tag(&map[0], TICKS_KEY), None, "OSM's way round stays open");
    }

    #[test]
    fn smoothing_takes_out_the_zigzag_and_keeps_the_ends() {
        // A face 100 m long with niches 2 m deep every 4 m, as OSM traces it.
        let zigzag: LineString<f64> = (0..=50)
            .map(|i| Coord {
                x: f64::from(i) * 2.0,
                y: if i % 2 == 0 { 0.0 } else { 2.0 },
            })
            .collect();
        // 0.3 mm at 1:10 000 with symbols at 150 %: 4.5 m.
        let smoothed = smooth(&zigzag, 4.5, false);
        assert_eq!(smoothed.0.first(), zigzag.0.first());
        assert_eq!(smoothed.0.last(), zigzag.0.last());
        let inner = &smoothed.0[5..smoothed.0.len() - 5];
        assert!(inner.iter().all(|c| (c.y - 1.0).abs() < 0.1), "{inner:?}");
        // A bend of the face as long as the face stays.
        let bend: LineString<f64> = vec![(0.0, 0.0), (50.0, 0.0), (50.0, 50.0)].into();
        let smoothed = smooth(&bend, 4.5, false);
        let corner = smoothed
            .0
            .iter()
            .map(|c| (c.x - 50.0).hypot(c.y))
            .fold(f64::INFINITY, f64::min);
        assert!(corner < 3.0, "the corner is rounded, not cut off: {corner}");
    }

    #[test]
    fn a_sprint_map_keeps_its_cliffs() {
        let f = fixture(MapKind::Sprint);
        let ground = terrain(&f, terraces(&[(0.0, 1.0)]));
        let mut map = vec![
            cliff(&f, &[(-50.0, 0.0), (50.0, 0.0)]),
            cliff(&f, &[(-50.0, 2.0), (50.0, 2.0)]),
        ];
        let (candidates, rock) = run(&f, &mut map, Some(&ground));
        assert_eq!(map.len(), 2);
        assert!(candidates.is_empty() && rock.is_empty());
    }
}
