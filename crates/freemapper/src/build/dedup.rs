//! Duplicate candidates of several sources (ADR 0047, 0051, 0055): the same wall, tree or lamp
//! from the cadastre, a register, the survey and OSM is kept once. The rank comes from
//! `freemapper:source`: register > ALKIS > survey > OSM ([`Rank`]). The source that loses is
//! named in the winner's `freemapper:also` (trees excepted), so a difference in currency stays
//! visible.

use std::collections::HashMap;

use crate::omap::{MapObject, MapObjectKind, SymbolSet};

/// How much a source is trusted for a duplicate, lowest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    /// OpenStreetMap (`osm`).
    Osm,
    /// The Berlin Straßenbefahrung (`geoportal-berlin/sb-…`).
    Survey,
    /// The cadastre: `alkis`, `alkis-nas/…` and the other Geoportal Berlin layers.
    Alkis,
    /// The registers: the Berlin tree, lighting and fountain registers ([`BERLIN_REGISTERS`]) and
    /// the tree cadastres (`trees/…`).
    Register,
}

/// The Geoportal Berlin layers that are registers: the tree cadastre, the lighting cadastre and
/// the fountain registers.
const BERLIN_REGISTERS: [&str; 5] = [
    "strassenbaeume",
    "anlagenbaeume",
    "beleuchtung",
    "zierbrunnen",
    "trinkwasserbrunnen",
];

/// The rank of an object by its `freemapper:source`; `None` for an object without one or from
/// a source that takes no part in the duplicates (land use, contours).
pub fn rank(object: &MapObject) -> Option<Rank> {
    let source = object
        .tags
        .iter()
        .find(|(k, _)| k == "freemapper:source")
        .map(|(_, v)| v.as_str())?;
    if source == "osm" {
        return Some(Rank::Osm);
    }
    if source == "alkis" || source.starts_with("alkis-nas/") {
        return Some(Rank::Alkis);
    }
    if source.starts_with("trees/") {
        return Some(Rank::Register);
    }
    let layer = source.strip_prefix("geoportal-berlin/")?;
    Some(if layer.starts_with("sb-") {
        Rank::Survey
    } else if BERLIN_REGISTERS.contains(&layer) {
        Rank::Register
    } else {
        Rank::Alkis
    })
}

/// A tree candidate this close to one of a higher rank is the same tree and is dropped.
const TREE_DEDUP_M: f64 = 2.0;
/// A line candidate with at least this share of its length within [`LINE_DEDUP_M`] of a line of a
/// higher rank with the same symbol is the same object and is dropped.
const LINE_DEDUP_SHARE: f64 = 0.8;
const LINE_DEDUP_M: f64 = 1.5;
/// The losing line is measured in pieces of at most this length (ground metres).
const LINE_DEDUP_STEP_M: f64 = 0.5;
/// Wall, retaining wall, fence, hedge, brook and ditch: the line symbols several sources give.
const LINE_DEDUP_CODES: [&str; 6] = ["515", "513.2", "518", "410.1", "305", "306"];
/// The tree symbols (large and small tree): every source draws its trees as one of them.
const TREE_CODES: [&str; 2] = ["417", "418"];

/// Drops the survey lines ([`LINE_DEDUP_CODES`]: `sb-mauer`, `sb-gelaender`) with at least
/// [`LINE_DEDUP_SHARE`] of their length within [`LINE_DEDUP_M`] of a `cadastre` line of a higher
/// rank with the same symbol, and the winner names them in `freemapper:also`
/// (`geoportal-berlin/sb-…/<gis_id>`). Returns the other objects in their order, then the kept
/// survey objects. No register has lines, so ALKIS lines never lose here. Run
/// [`drop_osm_lines_near_cadastre`] on the result.
pub fn drop_cadastre_lines_by_rank(
    cadastre: Vec<MapObject>,
    symbol_set: &SymbolSet,
    scale_denominator: u32,
) -> Vec<MapObject> {
    let (survey, mut others): (Vec<_>, Vec<_>) = cadastre
        .into_iter()
        .partition(|o| rank(o) == Some(Rank::Survey));
    let mut kept = drop_lines_near(
        survey,
        &mut others,
        |o| rank(o) > Some(Rank::Survey),
        symbol_set,
        scale_denominator,
    );
    others.append(&mut kept);
    others
}

/// Drops the OSM line candidates ([`LINE_DEDUP_CODES`]) with at least [`LINE_DEDUP_SHARE`] of
/// their length within [`LINE_DEDUP_M`] of a `cadastre` line (ALKIS, NAS or survey) with the same
/// symbol: the cadastre surveyed them. The `cadastre` object nearest to most of the dropped line
/// names it in `freemapper:also` (`osm/way/…`, several joined by `; `).
pub fn drop_osm_lines_near_cadastre(
    osm: Vec<MapObject>,
    cadastre: &mut [MapObject],
    symbol_set: &SymbolSet,
    scale_denominator: u32,
) -> Vec<MapObject> {
    drop_lines_near(
        osm,
        cadastre,
        |o| rank(o) > Some(Rank::Osm),
        symbol_set,
        scale_denominator,
    )
}

/// Adds `also` to the `freemapper:also` tag (`; `-separated, no duplicates).
pub(crate) fn add_also(tags: &mut Vec<(String, String)>, also: &str) {
    match tags.iter_mut().find(|(k, _)| k == "freemapper:also") {
        Some((_, v)) if !v.split("; ").any(|x| x == also) => {
            v.push_str("; ");
            v.push_str(also);
        }
        Some(_) => {}
        None => tags.push(("freemapper:also".to_string(), also.to_string())),
    }
}

/// Drops the `losers` lines ([`LINE_DEDUP_CODES`]) that follow a line with the same symbol among
/// the `winners` for which `is_winner` holds, and names them in that winner's `freemapper:also`.
fn drop_lines_near(
    losers: Vec<MapObject>,
    winners: &mut [MapObject],
    is_winner: impl Fn(&MapObject) -> bool,
    symbol_set: &SymbolSet,
    scale_denominator: u32,
) -> Vec<MapObject> {
    let codes: Vec<u32> = LINE_DEDUP_CODES
        .iter()
        .filter_map(|c| symbol_set.id_for_code(c))
        .collect();
    let line = |o: &MapObject| {
        matches!(o.kind, MapObjectKind::Path) && o.coords.len() >= 2 && codes.contains(&o.symbol_id)
    };
    let xy = |c: &crate::omap::MapCoord| (f64::from(c.x), f64::from(c.y));
    // Map coordinates are 1/1000 mm on paper: one ground metre is 10^6 / scale of them.
    let metre = 1e6 / f64::from(scale_denominator);
    let limit = LINE_DEDUP_M * metre;
    let cell = |(x, y): (f64, f64)| ((x / limit).floor() as i64, (y / limit).floor() as i64);
    // The winning segments by symbol, in a grid of `limit`-sized cells along each segment.
    type Segment = (usize, (f64, f64), (f64, f64));
    let mut segments: Vec<(u32, Segment)> = Vec::new();
    let mut grid: HashMap<(u32, i64, i64), Vec<usize>> = HashMap::new();
    for (index, object) in winners.iter().enumerate() {
        if !(line(object) && is_winner(object)) {
            continue;
        }
        for pair in object.coords.windows(2) {
            let (a, b) = (xy(&pair[0]), xy(&pair[1]));
            let id = segments.len();
            segments.push((object.symbol_id, (index, a, b)));
            let steps = ((b.0 - a.0).hypot(b.1 - a.1) / (limit / 2.0))
                .ceil()
                .max(1.0) as usize;
            let mut cells: Vec<(i64, i64)> = (0..=steps)
                .map(|i| {
                    let t = i as f64 / steps as f64;
                    cell((a.0 + t * (b.0 - a.0), a.1 + t * (b.1 - a.1)))
                })
                .collect();
            cells.dedup();
            for (cx, cy) in cells {
                let list = grid.entry((object.symbol_id, cx, cy)).or_default();
                if list.last() != Some(&id) {
                    list.push(id);
                }
            }
        }
    }
    if segments.is_empty() {
        return losers;
    }
    let distance = |p: (f64, f64), a: (f64, f64), b: (f64, f64)| {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let length2 = dx * dx + dy * dy;
        let t = if length2 > 0.0 {
            (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / length2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (p.0 - a.0 - t * dx).hypot(p.1 - a.1 - t * dy)
    };
    // The winner nearest to `p` within `limit`, among the segments with `symbol`. A segment
    // within `limit` of `p` has a sample cell within two cells of `p`'s.
    let nearest = |symbol: u32, p: (f64, f64)| -> Option<usize> {
        let (cx, cy) = cell(p);
        let mut best: Option<(f64, usize)> = None;
        for i in cx - 2..=cx + 2 {
            for j in cy - 2..=cy + 2 {
                for &id in grid.get(&(symbol, i, j)).into_iter().flatten() {
                    let (_, (index, a, b)) = segments[id];
                    let d = distance(p, a, b);
                    if d <= limit && best.is_none_or(|(e, _)| d < e) {
                        best = Some((d, index));
                    }
                }
            }
        }
        best.map(|(_, index)| index)
    };
    let step = LINE_DEDUP_STEP_M * metre;
    let mut kept = Vec::with_capacity(losers.len());
    for object in losers {
        if !line(&object) {
            kept.push(object);
            continue;
        }
        // Length near each winner, from the middle of pieces at most `step` long.
        let mut near: HashMap<usize, f64> = HashMap::new();
        let mut total = 0.0;
        for pair in object.coords.windows(2) {
            let (a, b) = (xy(&pair[0]), xy(&pair[1]));
            let length = (b.0 - a.0).hypot(b.1 - a.1);
            let pieces = (length / step).ceil().max(1.0) as usize;
            let piece = length / pieces as f64;
            total += length;
            for i in 0..pieces {
                let t = (i as f64 + 0.5) / pieces as f64;
                let p = (a.0 + t * (b.0 - a.0), a.1 + t * (b.1 - a.1));
                if let Some(index) = nearest(object.symbol_id, p) {
                    *near.entry(index).or_default() += piece;
                }
            }
        }
        let covered: f64 = near.values().sum();
        if total <= 0.0 || covered < LINE_DEDUP_SHARE * total {
            kept.push(object);
            continue;
        }
        let winner = near
            .into_iter()
            .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
            .map(|(index, _)| index)
            .expect("some length is near");
        // `osm/way/1`, `geoportal-berlin/sb-gelaender/Li_…`: the source and the id in it.
        let tag = |key: &str| {
            object
                .tags
                .iter()
                .find(|(k, _)| k == key)
                .map_or_else(String::new, |(_, v)| v.clone())
        };
        let also = format!(
            "{}/{}",
            tag("freemapper:source"),
            tag("freemapper:source_id")
        );
        add_also(&mut winners[winner].tags, &also);
    }
    kept
}

/// Drops the tree points (the [`TREE_CODES`] symbols) within [`TREE_DEDUP_M`] of a tree point of
/// a higher rank: an OSM `natural=tree` at a register or cadastre tree, a NAS tree at a register
/// tree. The register knows the crown, so it wins. Everything else stays, in its order. Returns
/// the OSM and the `cadastre` candidates.
pub fn drop_trees_by_rank(
    osm: Vec<MapObject>,
    cadastre: Vec<MapObject>,
    symbol_set: &SymbolSet,
    scale_denominator: u32,
) -> (Vec<MapObject>, Vec<MapObject>) {
    let codes: Vec<u32> = TREE_CODES
        .iter()
        .filter_map(|c| symbol_set.id_for_code(c))
        .collect();
    /// A tree point (map coordinates) and its rank.
    type Tree = ((f64, f64), Rank);
    let tree = |o: &MapObject| -> Option<Tree> {
        match (&o.kind, o.coords.first()) {
            (MapObjectKind::Point { .. }, Some(c)) if codes.contains(&o.symbol_id) => {
                Some(((f64::from(c.x), f64::from(c.y)), rank(o)?))
            }
            _ => None,
        }
    };
    let trees: Vec<Tree> = cadastre.iter().filter_map(tree).collect();
    if trees.is_empty() {
        return (osm, cadastre);
    }
    // Map coordinates are 1/1000 mm on paper: one ground metre is 10^6 / scale of them.
    let limit = TREE_DEDUP_M * 1e6 / f64::from(scale_denominator);
    // A grid of `limit`-sized cells: a tree within `limit` lies in the same or a neighbouring cell.
    let cell = |(x, y): (f64, f64)| ((x / limit).floor() as i64, (y / limit).floor() as i64);
    let mut grid: HashMap<(i64, i64), Vec<Tree>> = HashMap::new();
    for t in trees {
        grid.entry(cell(t.0)).or_default().push(t);
    }
    let outranked = |o: &MapObject| {
        tree(o).is_some_and(|(p @ (x, y), own)| {
            let (cx, cy) = cell(p);
            (cx - 1..=cx + 1).any(|i| {
                (cy - 1..=cy + 1).any(|j| {
                    grid.get(&(i, j)).is_some_and(|ts| {
                        ts.iter()
                            .any(|((tx, ty), r)| *r > own && (x - tx).hypot(y - ty) <= limit)
                    })
                })
            })
        })
    };
    let osm = osm.into_iter().filter(|o| !outranked(o)).collect();
    let cadastre = cadastre.into_iter().filter(|o| !outranked(o)).collect();
    (osm, cadastre)
}

/// An OSM street lamp this close (ground metres) to a lamp of the lighting cadastre is the same
/// lamp. OSM lamps lie up to 6.7 m off the cadastre ones (live run, Task 5b), and lamps stand 20
/// to 30 m apart, so 5 m stays well under half the spacing.
const OSM_LAMP_DEDUP_M: f64 = 5.0;
/// A survey mast this close to a cadastre lamp is the lamp, and an OSM hydrant, bench, mast or
/// play item this close to the survey one is that one. Survey masts 2 m or more from a lamp are
/// mostly sign and signal masts.
const FURNITURE_DEDUP_M: f64 = 2.0;
/// The same for a fountain or drinking-water point of the fountain registers.
const FOUNTAIN_DEDUP_M: f64 = 3.0;

/// One kind of winning point and the points that give way to it.
struct PointDedup {
    /// `(tag key, value)` of the winners: a register's or the full ALKIS's `freemapper:source`
    /// (a value ending in `*` is a prefix) or a survey type's `berlin:layer`.
    winner: (&'static str, &'static str),
    /// The symbol a winner must have; `None`: any.
    winner_code: Option<&'static str>,
    /// `(tag key, value, limit in ground metres)` of the losers: an OSM rule (`freemapper:osm`,
    /// one of its `; `-separated parts; a value ending in `*` is a prefix) or a survey type
    /// (`berlin:layer`).
    losers: &'static [(&'static str, &'static str, f64)],
}

/// In order: the registers beat the survey and OSM, the full ALKIS fountains beat OSM, then the
/// survey furniture (schoolyard only) beats OSM. A point dropped by an earlier entry wins nothing
/// later.
const POINT_DEDUPS: [PointDedup; 9] = [
    PointDedup {
        winner_code: None,
        winner: ("freemapper:source", "geoportal-berlin/beleuchtung"),
        losers: &[
            ("freemapper:osm", "highway=street_lamp", OSM_LAMP_DEDUP_M),
            ("berlin:layer", "as_mast", FURNITURE_DEDUP_M),
        ],
    },
    PointDedup {
        winner_code: None,
        winner: ("freemapper:source", "geoportal-berlin/zierbrunnen"),
        losers: &[
            ("freemapper:osm", "amenity=fountain", FOUNTAIN_DEDUP_M),
            ("berlin:layer", "af_wasserpumpen_brunnen", FOUNTAIN_DEDUP_M),
            (
                "berlin:layer",
                "bv_springbrunnen_zierbrunnen",
                FOUNTAIN_DEDUP_M,
            ),
        ],
    },
    PointDedup {
        winner_code: None,
        winner: ("freemapper:source", "geoportal-berlin/trinkwasserbrunnen"),
        losers: &[
            ("freemapper:osm", "amenity=drinking_water", FOUNTAIN_DEDUP_M),
            (
                "berlin:layer",
                "ac_trinkwasserbrunnen_wasserspender",
                FOUNTAIN_DEDUP_M,
            ),
        ],
    },
    // The full ALKIS fountains and wells (311) beat the OSM ones. OSM has no monument rule, so a
    // NAS 526 has nothing to beat.
    PointDedup {
        winner_code: Some("311"),
        winner: ("freemapper:source", "alkis-nas/*"),
        losers: &[
            ("freemapper:osm", "amenity=fountain", FOUNTAIN_DEDUP_M),
            ("freemapper:osm", "man_made=water_well", FOUNTAIN_DEDUP_M),
        ],
    },
    PointDedup {
        winner_code: None,
        winner: ("berlin:layer", "aq_hydrant"),
        losers: &[(
            "freemapper:osm",
            "emergency=fire_hydrant",
            FURNITURE_DEDUP_M,
        )],
    },
    PointDedup {
        winner_code: None,
        winner: ("berlin:layer", "bj_sitzbank"),
        losers: &[("freemapper:osm", "amenity=bench", FURNITURE_DEDUP_M)],
    },
    PointDedup {
        winner_code: None,
        winner: ("berlin:layer", "as_mast"),
        losers: SURVEY_MAST_LOSERS,
    },
    PointDedup {
        winner_code: None,
        winner: ("berlin:layer", "am_fahnenmast"),
        losers: SURVEY_MAST_LOSERS,
    },
    PointDedup {
        winner_code: None,
        winner: ("berlin:layer", "aw_spielgeraet"),
        losers: &[("freemapper:osm", "playground=*", FURNITURE_DEDUP_M)],
    },
];

/// A survey mast or flag pole and an OSM one at the same place are one pole, whichever word
/// each source uses.
const SURVEY_MAST_LOSERS: &[(&str, &str, f64)] = &[
    ("freemapper:osm", "man_made=mast", FURNITURE_DEDUP_M),
    ("freemapper:osm", "man_made=flagpole", FURNITURE_DEDUP_M),
];

/// Whether the tag value `value` holds `wanted`: one of its `; `-separated parts equals it, or
/// starts with it when `wanted` ends in `*`.
fn tag_holds(value: &str, wanted: &str) -> bool {
    value
        .split("; ")
        .any(|part| match wanted.strip_suffix('*') {
            Some(prefix) => part.starts_with(prefix),
            None => part == wanted,
        })
}

/// The registers win over the survey and OSM, and the survey furniture over OSM
/// ([`POINT_DEDUPS`]): an OSM street lamp within [`OSM_LAMP_DEDUP_M`] or a survey mast within
/// [`FURNITURE_DEDUP_M`] of a lamp of the lighting cadastre, an OSM or survey fountain (drinking
/// water) within [`FOUNTAIN_DEDUP_M`] of a point of the fountain (drinking water) register, an
/// OSM fountain or well within [`FOUNTAIN_DEDUP_M`] of a full ALKIS (NAS) one, and
/// an OSM hydrant, bench, mast, flag pole or play item within [`FURNITURE_DEDUP_M`] of the
/// survey one are dropped. A line (bench, play item) counts by the middle of its extent. The
/// winner names each dropped object in `freemapper:also` (`osm/node/1`,
/// `geoportal-berlin/sb-moebel/<gis_id>`). Returns the OSM and the other (`cadastre`: cadastre,
/// register and survey) candidates without them.
pub fn drop_points_near_registers(
    osm: Vec<MapObject>,
    mut cadastre: Vec<MapObject>,
    symbol_set: &SymbolSet,
    scale_denominator: u32,
) -> (Vec<MapObject>, Vec<MapObject>) {
    let tag = |o: &MapObject, key: &str| -> Option<String> {
        o.tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    };
    let holds = |o: &MapObject, (key, value): (&str, &str)| {
        tag(o, key).is_some_and(|v| tag_holds(&v, value))
    };
    // A point, or the middle of a short line's extent (benches, play equipment).
    let position = |o: &MapObject| match (&o.kind, o.coords.first()) {
        (MapObjectKind::Point { .. }, Some(c)) => Some((f64::from(c.x), f64::from(c.y))),
        (MapObjectKind::Path, Some(_)) => {
            let xs = o.coords.iter().map(|c| f64::from(c.x));
            let ys = o.coords.iter().map(|c| f64::from(c.y));
            let (min_x, max_x) = xs.fold((f64::MAX, f64::MIN), |(a, b), x| (a.min(x), b.max(x)));
            let (min_y, max_y) = ys.fold((f64::MAX, f64::MIN), |(a, b), y| (a.min(y), b.max(y)));
            Some(((min_x + max_x) / 2.0, (min_y + max_y) / 2.0))
        }
        _ => None,
    };
    // Map coordinates are 1/1000 mm on paper: one ground metre is 10^6 / scale of them.
    let metre = 1e6 / f64::from(scale_denominator);
    let mut dropped_osm = vec![false; osm.len()];
    let mut dropped_cadastre = vec![false; cadastre.len()];
    let mut also: Vec<(usize, String)> = Vec::new();
    for rule in &POINT_DEDUPS {
        let winners: Vec<(usize, (f64, f64))> = cadastre
            .iter()
            .enumerate()
            .filter(|&(i, o)| !dropped_cadastre[i] && holds(o, rule.winner))
            .filter(|(_, o)| {
                rule.winner_code
                    .is_none_or(|code| symbol_set.id_for_code(code) == Some(o.symbol_id))
            })
            .filter_map(|(i, o)| position(o).map(|p| (i, p)))
            .collect();
        if winners.is_empty() {
            continue;
        }
        let mut check = |object: &MapObject, gone: &mut bool| {
            let Some(limit_m) = rule
                .losers
                .iter()
                .find(|&&(key, value, _)| holds(object, (key, value)))
                .map(|&(_, _, limit_m)| limit_m)
            else {
                return;
            };
            let Some((x, y)) = position(object).filter(|_| !*gone) else {
                return;
            };
            let limit = limit_m * metre;
            let nearest = winners
                .iter()
                .map(|&(i, (wx, wy))| (i, (x - wx).hypot(y - wy)))
                .filter(|&(_, d)| d <= limit)
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((winner, _)) = nearest {
                *gone = true;
                let name = format!(
                    "{}/{}",
                    tag(object, "freemapper:source").unwrap_or_default(),
                    tag(object, "freemapper:source_id").unwrap_or_default()
                );
                also.push((winner, name));
            }
        };
        for (object, gone) in osm.iter().zip(&mut dropped_osm) {
            check(object, gone);
        }
        for (object, gone) in cadastre.iter().zip(&mut dropped_cadastre) {
            check(object, gone);
        }
    }
    for (winner, name) in also {
        add_also(&mut cadastre[winner].tags, &name);
    }
    let keep = |objects: Vec<MapObject>, dropped: Vec<bool>| -> Vec<MapObject> {
        objects
            .into_iter()
            .zip(dropped)
            .filter_map(|(o, gone)| (!gone).then_some(o))
            .collect()
    };
    (keep(osm, dropped_osm), keep(cadastre, dropped_cadastre))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::omap::{MapCoord, MapObjectKind};
    use crate::project::MapKind;

    /// Map coordinates of one ground metre at 1:4000.
    const M: i32 = 250;
    const SCALE: u32 = 4000;

    fn set() -> SymbolSet {
        // The schoolyard set has the furniture (544, 545) and every line of the dedup.
        MapKind::Schoolyard.symbol_set()
    }

    fn object(
        code: &str,
        kind: MapObjectKind,
        coords: Vec<MapCoord>,
        tags: &[(&str, &str)],
    ) -> MapObject {
        MapObject {
            symbol_id: set().id_for_code(code).unwrap(),
            kind,
            coords,
            tags: tags
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    /// A 20 m line from east to west at `dn` metres north, from `source`.
    fn line(code: &str, source: &str, id: &str, dn: f64, extra: &[(&str, &str)]) -> MapObject {
        let y = (dn * f64::from(M)) as i32;
        let mut tags = vec![("freemapper:source", source), ("freemapper:source_id", id)];
        tags.extend_from_slice(extra);
        object(
            code,
            MapObjectKind::Path,
            vec![MapCoord::new(0, y), MapCoord::new(20 * M, y)],
            &tags,
        )
    }

    fn point(code: &str, source: &str, id: &str, de: f64, extra: &[(&str, &str)]) -> MapObject {
        let mut tags = vec![("freemapper:source", source), ("freemapper:source_id", id)];
        tags.extend_from_slice(extra);
        object(
            code,
            MapObjectKind::Point { rotation_rad: 0.0 },
            vec![MapCoord::new((de * f64::from(M)) as i32, 0)],
            &tags,
        )
    }

    fn tag<'a>(object: &'a MapObject, key: &str) -> Option<&'a str> {
        object
            .tags
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    fn ids(objects: &[MapObject]) -> Vec<&str> {
        objects
            .iter()
            .map(|o| tag(o, "freemapper:source_id").unwrap())
            .collect()
    }

    #[test]
    fn dedup_rank_from_provenance() {
        let ranked = |source: &str| rank(&point("418", source, "1", 0.0, &[]));
        for (source, expected) in [
            ("osm", Rank::Osm),
            ("geoportal-berlin/sb-mauer", Rank::Survey),
            ("geoportal-berlin/sb-moebel", Rank::Survey),
            ("alkis", Rank::Alkis),
            ("alkis-nas/sonstiges-bauwerk", Rank::Alkis),
            ("geoportal-berlin/bauwerke-linien", Rank::Alkis),
            ("geoportal-berlin/building-parts", Rank::Alkis),
            ("geoportal-berlin/strassenbaeume", Rank::Register),
            ("geoportal-berlin/anlagenbaeume", Rank::Register),
            ("geoportal-berlin/beleuchtung", Rank::Register),
            ("geoportal-berlin/zierbrunnen", Rank::Register),
            ("geoportal-berlin/trinkwasserbrunnen", Rank::Register),
            ("trees/potsdam", Rank::Register),
        ] {
            assert_eq!(ranked(source), Some(expected), "{source}");
        }
        assert_eq!(ranked("land-use"), None);
        let mut bare = point("418", "osm", "1", 0.0, &[]);
        bare.tags.clear();
        assert_eq!(rank(&bare), None);
        assert!(Rank::Register > Rank::Alkis && Rank::Alkis > Rank::Survey);
        assert!(Rank::Survey > Rank::Osm);
    }

    #[test]
    fn osm_lines_near_nas_lines_are_dropped() {
        let nas = "alkis-nas/sonstiges-bauwerk";
        let mut cadastre = vec![
            line("518", nas, "DEBBAL1", 0.0, &[]),
            line(
                "306",
                "alkis-nas/untergeordnetes-gewaesser",
                "DEBBAL2",
                10.0,
                &[],
            ),
        ];
        let osm = vec![
            // A fence 0.5 m off the NAS fence and a ditch 1 m off the NAS ditch.
            line(
                "518",
                "osm",
                "way/1",
                0.5,
                &[("freemapper:osm", "barrier=fence")],
            ),
            line(
                "306",
                "osm",
                "way/2",
                11.0,
                &[("freemapper:osm", "waterway=ditch")],
            ),
            // A wall along the fence: another symbol, it stays.
            line(
                "515",
                "osm",
                "way/3",
                0.5,
                &[("freemapper:osm", "barrier=wall")],
            ),
            // A fence 3 m off: too far.
            line(
                "518",
                "osm",
                "way/4",
                3.0,
                &[("freemapper:osm", "barrier=fence")],
            ),
        ];
        let kept = drop_osm_lines_near_cadastre(osm, &mut cadastre, &set(), SCALE);
        assert_eq!(ids(&kept), ["way/3", "way/4"]);
        assert_eq!(tag(&cadastre[0], "freemapper:also"), Some("osm/way/1"));
        assert_eq!(tag(&cadastre[1], "freemapper:also"), Some("osm/way/2"));
        // A vereinfacht passage (522) is no line of the dedup.
        let mut passages = vec![line("522", "alkis", "DEBBAL3", 0.0, &[])];
        let osm = vec![line("522", "osm", "way/5", 0.0, &[])];
        assert_eq!(
            drop_osm_lines_near_cadastre(osm, &mut passages, &set(), SCALE).len(),
            1
        );
    }

    #[test]
    fn nas_fountain_beats_the_osm_fountain() {
        let nas = "alkis-nas/sonstiges-bauwerk";
        let cadastre = vec![
            point(
                "311",
                nas,
                "DEBBAL1",
                0.0,
                &[("freemapper:alkis", "Springbrunnen, Zierbrunnen")],
            ),
            // A monument is no fountain: the OSM fountain beside it stays.
            point("526", nas, "DEBBAL2", 100.0, &[]),
        ];
        let osm = vec![
            point(
                "311",
                "osm",
                "node/1",
                1.0,
                &[("freemapper:osm", "amenity=fountain")],
            ),
            point(
                "311",
                "osm",
                "node/2",
                -2.5,
                &[("freemapper:osm", "man_made=water_well")],
            ),
            // 5 m off: another fountain.
            point(
                "311",
                "osm",
                "node/3",
                5.0,
                &[("freemapper:osm", "amenity=fountain")],
            ),
            point(
                "311",
                "osm",
                "node/4",
                101.0,
                &[("freemapper:osm", "amenity=fountain")],
            ),
        ];
        let (osm, cadastre) = drop_points_near_registers(osm, cadastre, &set(), SCALE);
        assert_eq!(ids(&osm), ["node/3", "node/4"]);
        assert_eq!(ids(&cadastre), ["DEBBAL1", "DEBBAL2"]);
        assert_eq!(
            tag(&cadastre[0], "freemapper:also"),
            Some("osm/node/1; osm/node/2")
        );
        assert_eq!(tag(&cadastre[1], "freemapper:also"), None);
    }

    /// The whole Berlin chain on one scene: ALKIS wins over the survey, both over OSM, the
    /// registers over the survey and OSM points. The expected ids, order and `freemapper:also`
    /// texts were confirmed against the functions before the move (a721b8b:
    /// `drop_survey_lines_near_alkis`, `drop_osm_trees_near_cadastre`, `drop_osm_lines_near_alkis`,
    /// `drop_points_near_registers` in `build/berlin.rs`), on the same scene.
    #[test]
    fn berlin_dedup_unchanged() {
        let alkis = "geoportal-berlin/bauwerke-linien";
        let cadastre = vec![
            line(
                "515",
                "geoportal-berlin/sb-mauer",
                "Li_1",
                0.3,
                &[("berlin:layer", "bh_mauer")],
            ),
            line("515", alkis, "DEBE1", 0.0, &[("freemapper:alkis", "Mauer")]),
            // A survey railing on its own: no ALKIS line, it stays and beats the OSM fence.
            line(
                "518",
                "geoportal-berlin/sb-gelaender",
                "Li_2",
                30.0,
                &[("berlin:layer", "bf_gelaender")],
            ),
            point(
                "418",
                "geoportal-berlin/strassenbaeume",
                "baum1",
                100.0,
                &[],
            ),
            point("545", "geoportal-berlin/beleuchtung", "lamp1", 200.0, &[]),
            point(
                "544",
                "geoportal-berlin/sb-objekte",
                "mast1",
                201.0,
                &[("berlin:layer", "as_mast")],
            ),
            line("522", "geoportal-berlin/building-parts", "DEBE2", 50.0, &[]),
        ];
        let osm = vec![
            line(
                "515",
                "osm",
                "way/1",
                0.6,
                &[("freemapper:osm", "barrier=wall")],
            ),
            line(
                "518",
                "osm",
                "way/2",
                30.5,
                &[("freemapper:osm", "barrier=fence")],
            ),
            point(
                "418",
                "osm",
                "node/1",
                101.0,
                &[("freemapper:osm", "natural=tree")],
            ),
            point(
                "418",
                "osm",
                "node/2",
                110.0,
                &[("freemapper:osm", "natural=tree")],
            ),
            point(
                "545",
                "osm",
                "node/3",
                203.0,
                &[("freemapper:osm", "highway=street_lamp")],
            ),
        ];
        let set = set();
        let mut cadastre = drop_cadastre_lines_by_rank(cadastre, &set, SCALE);
        let (osm, cadastre_after_trees) = drop_trees_by_rank(osm, cadastre.clone(), &set, SCALE);
        assert_eq!(cadastre_after_trees, cadastre);
        let osm = drop_osm_lines_near_cadastre(osm, &mut cadastre, &set, SCALE);
        let (osm, cadastre) = drop_points_near_registers(osm, cadastre, &set, SCALE);
        // The order: the cadastre first, then the kept survey objects.
        assert_eq!(ids(&cadastre), ["DEBE1", "baum1", "lamp1", "DEBE2", "Li_2"]);
        assert_eq!(ids(&osm), ["node/2"]);
        assert_eq!(
            tag(&cadastre[0], "freemapper:also"),
            Some("geoportal-berlin/sb-mauer/Li_1; osm/way/1")
        );
        assert_eq!(
            tag(&cadastre[2], "freemapper:also"),
            Some("osm/node/3; geoportal-berlin/sb-objekte/mast1")
        );
        assert_eq!(tag(&cadastre[4], "freemapper:also"), Some("osm/way/2"));
        assert_eq!(tag(&cadastre[1], "freemapper:also"), None);
    }
}
