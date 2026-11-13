//! Duplicate candidates of several sources (ADR 0047, 0051, 0055): the same wall, tree or lamp
//! from the cadastre, a register, the survey and OSM is kept once. The rank comes from
//! `freemapper:source`: register > ALKIS > survey > OSM ([`Rank`]). The source that loses is
//! named in the winner's `freemapper:also` (trees excepted), so a difference in currency stays
//! visible.

use std::collections::HashMap;

use crate::build::geometry::object_polygon;
use crate::measure::{PaperMm, map_units_per_metre};
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
    /// The registers: the Berlin tree, lighting and fountain registers (those the Berlin adapter
    /// names, `freemapper_adapters_de::berlin::is_register`) and the tree cadastres (`trees/…`).
    Register,
}

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
    if source == "alkis" || source == "alkis-nutzung" || source.starts_with("alkis-nas/") {
        return Some(Rank::Alkis);
    }
    if source.starts_with("trees/") {
        return Some(Rank::Register);
    }
    let layer = source.strip_prefix("geoportal-berlin/")?;
    Some(if layer.starts_with("sb-") {
        Rank::Survey
    } else if freemapper_adapters_de::berlin::is_register(layer) {
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
/// Wall, retaining wall, fence, hedge, brook, ditch, kerb, embankment top and stairway: the line symbols
/// several sources give.
const LINE_DEDUP_CODES: [&str; 12] = [
    "515", "513.2", "518", "410.1", "305", "306", "501.1", "104", "532.6", "532.7", "532.8",
    "532.9",
];
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
    let metre = map_units_per_metre(scale_denominator);
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
    let limit = TREE_DEDUP_M * map_units_per_metre(scale_denominator);
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

/// The area symbols several sources give, and the share of a lower-ranked area that has to lie
/// under higher-ranked areas of the same symbol for it to be the same object: passages and
/// canopies (522; an OSM way is a strip of a default width, so half of it is enough), the areas
/// that shall not be entered (520; an OSM strip along a private road: the rest of it, outside
/// the land use area, is news, so 80 %) and bridges (501.5; see [`area_may_lose`]).
const AREA_DEDUPS: [(&str, f64); 3] = [("522", 0.5), ("520", 0.8), ("501.5", 0.5)];
/// The agreement of two areas is judged with the higher-ranked one widened by this much on
/// paper (mm): one thin line width, at which two outlines of the same object still read as one.
/// A ruling (2026-11-10). At 1:4000 it is 1 m, at 1:5000 1.25 m.
pub const AREA_DEDUP_TOLERANCE_MM: PaperMm = PaperMm(0.25);

/// Whether `object`, which has the area symbol `code`, may be dropped as a duplicate: every
/// 522 and 520, but of the 501.5 only an OSM bridge, because the other 501.5 candidates are
/// paved ground of the walkable area that lies under bridges, paths and squares by design.
fn area_may_lose(code: &str, object: &MapObject) -> bool {
    code != "501.5"
        || object
            .tags
            .iter()
            .any(|(k, v)| k == "freemapper:osm" && tag_holds(v, "man_made=bridge"))
}

/// Whether `object` with the area symbol `code` may win over a duplicate: every 522 and 520, but
/// of the 501.5 only the ALKIS bridge decks of Berlin (`bauwerke-flaechen`).
fn area_may_win(code: &str, object: &MapObject) -> bool {
    code != "501.5"
        || object
            .tags
            .iter()
            .any(|(k, v)| k == "berlin:layer" && v == "bauwerke-flaechen")
}

/// Drops the `candidates` areas ([`AREA_DEDUPS`]: passages, canopies, areas that shall not be
/// entered, bridges) of which at least the listed share lies within [`AREA_DEDUP_TOLERANCE_MM`]
/// of the areas with the same symbol and a higher [`Rank`], among the `candidates` themselves
/// and the `others` (map objects such as the canopies, other candidates such as the ALKIS land
/// use areas). The share is the area of the dropped one under the union of the winners. The
/// winner under most of it names the dropped area in `freemapper:also`. Only an area that stays can
/// win: one dropped for a higher rank drops nothing (ADR 0078, amendment). Everything else stays,
/// in its order. Two areas of one rank never drop each other, whatever their overlap.
pub fn drop_areas_by_rank(
    candidates: Vec<MapObject>,
    others: &mut [&mut [MapObject]],
    symbol_set: &SymbolSet,
) -> Vec<MapObject> {
    use geo::algorithm::buffer::BufferStyle;
    use geo::{Area as _, BooleanOps as _, BoundingRect as _, Buffer as _, Intersects as _};
    type Bounds = geo_types::Rect<f64>;
    let codes: Vec<(u32, &str, f64)> = AREA_DEDUPS
        .iter()
        .filter_map(|&(code, share)| Some((symbol_set.id_for_code(code)?, code, share)))
        .collect();
    let code_of = |o: &MapObject| codes.iter().find(|(id, _, _)| *id == o.symbol_id).copied();
    let tolerance = AREA_DEDUP_TOLERANCE_MM.map_units();
    struct Winner {
        slot: usize,
        index: usize,
        symbol: u32,
        rank: Rank,
        polygon: geo_types::Polygon<f64>,
        bounds: Bounds,
    }
    let mut winners: Vec<Winner> = Vec::new();
    let mut add = |slot: usize, objects: &[MapObject]| {
        for (index, o) in objects.iter().enumerate() {
            let Some((symbol, code, _)) = code_of(o) else {
                continue;
            };
            let (Some(rank), true) = (rank(o), area_may_win(code, o)) else {
                continue;
            };
            let Some(polygon) = object_polygon(o) else {
                continue;
            };
            let Some(bounds) = polygon.bounding_rect() else {
                continue;
            };
            winners.push(Winner {
                slot,
                index,
                symbol,
                rank,
                polygon,
                bounds,
            });
        }
    };
    add(0, &candidates);
    for (i, objects) in others.iter().enumerate() {
        add(i + 1, objects);
    }
    if winners.is_empty() {
        return candidates;
    }
    let style = BufferStyle::new(tolerance);
    let mut gone = vec![false; candidates.len()];
    let mut also: Vec<(usize, usize, String)> = Vec::new();
    // Highest rank first: only an area that survives may drop a lower one, so a winner is
    // always decided before the areas it could drop.
    let mut order: Vec<usize> = (0..candidates.len()).collect();
    order.sort_by(|&a, &b| {
        let (ra, rb) = (rank(&candidates[a]), rank(&candidates[b]));
        rb.partial_cmp(&ra).unwrap_or(std::cmp::Ordering::Equal)
    });
    for i in order {
        let object = &candidates[i];
        let Some((symbol, code, share)) = code_of(object) else {
            continue;
        };
        let (Some(own), true) = (rank(object), area_may_lose(code, object)) else {
            continue;
        };
        let Some(polygon) = object_polygon(object) else {
            continue;
        };
        let (Some(bounds), area) = (polygon.bounding_rect(), polygon.unsigned_area()) else {
            continue;
        };
        if area <= 0.0 {
            continue;
        }
        let reach = Bounds::new(
            geo_types::coord! { x: bounds.min().x - tolerance, y: bounds.min().y - tolerance },
            geo_types::coord! { x: bounds.max().x + tolerance, y: bounds.max().y + tolerance },
        );
        let mut covering: Option<geo_types::MultiPolygon<f64>> = None;
        let mut best: Option<(f64, usize, usize)> = None;
        for w in &winners {
            if w.symbol != symbol
                || w.rank <= own
                || (w.slot == 0 && (w.index == i || gone[w.index]))
                || !w.bounds.intersects(&reach)
            {
                continue;
            }
            let widened = w.polygon.buffer_with_style(style.clone());
            let overlap = polygon.intersection(&widened).unsigned_area();
            if overlap <= 0.0 {
                continue;
            }
            if best.is_none_or(|(b, _, _)| overlap > b) {
                best = Some((overlap, w.slot, w.index));
            }
            covering = Some(match covering {
                Some(c) => c.union(&widened),
                None => widened,
            });
        }
        let (Some(covering), Some((_, slot, index))) = (covering, best) else {
            continue;
        };
        if polygon.intersection(&covering).unsigned_area() < share * area {
            continue;
        }
        gone[i] = true;
        let tag = |key: &str| {
            object
                .tags
                .iter()
                .find(|(k, _)| k == key)
                .map_or_else(String::new, |(_, v)| v.clone())
        };
        also.push((
            slot,
            index,
            format!(
                "{}/{}",
                tag("freemapper:source"),
                tag("freemapper:source_id")
            ),
        ));
    }
    let mut candidates = candidates;
    for (slot, index, name) in &also {
        let target = if *slot == 0 {
            &mut candidates[*index]
        } else {
            &mut others[*slot - 1][*index]
        };
        add_also(&mut target.tags, name);
    }
    candidates
        .into_iter()
        .zip(gone)
        .filter_map(|(o, gone)| (!gone).then_some(o))
        .collect()
}

/// An OSM street lamp this close (ground metres) to a lamp of the lighting cadastre is the same
/// lamp. OSM lamps lie up to 6.7 m off the cadastre ones (live run, Task 5b), and lamps stand 20
/// to 30 m apart, so 5 m stays well under half the spacing.
const OSM_LAMP_DEDUP_M: Limit = Limit::Metres(5.0);
/// A survey mast this close to a cadastre lamp is the lamp, and an OSM hydrant, bench, mast or
/// play item this close to the survey one is that one. Survey masts 2 m or more from a lamp are
/// mostly sign and signal masts.
const FURNITURE_DEDUP_M: Limit = Limit::Metres(2.0);
/// The same for the fountain, well and drinking-water points of two sources (ADR 0087): 0.75 mm
/// on paper (3 m at 1:4000, 3.75 m at 1:5000), a little under the 1.05 mm at which two 311
/// symbols touch, and never under 3 m on the ground (1:1000), because the sources place the same
/// fountain a few metres apart whatever the scale. The measured pairs lie at 0.05 to 0.5 m, the
/// next point of another source 9.8 m away.
const FOUNTAIN_DEDUP: Limit = Limit::Paper {
    mm: PaperMm(0.75),
    at_least_m: 3.0,
};

/// How close two points of different sources must be to count as one object.
#[derive(Clone, Copy)]
enum Limit {
    /// Ground metres.
    Metres(f64),
    /// Millimetres on paper, as what the printed map shows twice, but at least `at_least_m`
    /// ground metres: the positional error of the sources does not shrink with the scale.
    Paper { mm: PaperMm, at_least_m: f64 },
}

impl Limit {
    /// The limit in map coordinates (1/1000 mm on paper) at `scale_denominator`.
    fn map_units(self, scale_denominator: u32) -> f64 {
        let metre = map_units_per_metre(scale_denominator);
        match self {
            Self::Metres(m) => m * metre,
            Self::Paper { mm, at_least_m } => mm.map_units().max(at_least_m * metre),
        }
    }
}

/// One kind of winning point and the points that give way to it.
struct PointDedup {
    /// `(tag key, value)` of the winners: a register's or the full ALKIS's `freemapper:source`
    /// (a value ending in `*` is a prefix) or a survey type's `berlin:layer`.
    winner: (&'static str, &'static str),
    /// The symbol a winner must have; `None`: any.
    winner_code: Option<&'static str>,
    /// `(tag key, value, limit)` of the losers: an OSM rule (`freemapper:osm`,
    /// one of its `; `-separated parts; a value ending in `*` is a prefix) or a survey type
    /// (`berlin:layer`).
    losers: &'static [(&'static str, &'static str, Limit)],
}

/// In order: the registers beat the survey and OSM, the full ALKIS fountains beat OSM, then the
/// survey furniture beats OSM (on a sprint map it is the control sites, ADR 0077). A point dropped by an earlier entry wins nothing
/// later.
const POINT_DEDUPS: [PointDedup; 15] = [
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
            ("freemapper:osm", "amenity=fountain", FOUNTAIN_DEDUP),
            ("freemapper:osm", "man_made=water_well", FOUNTAIN_DEDUP),
            ("berlin:layer", "af_wasserpumpen_brunnen", FOUNTAIN_DEDUP),
            (
                "berlin:layer",
                "bv_springbrunnen_zierbrunnen",
                FOUNTAIN_DEDUP,
            ),
        ],
    },
    PointDedup {
        winner_code: None,
        winner: ("freemapper:source", "geoportal-berlin/trinkwasserbrunnen"),
        losers: &[
            ("freemapper:osm", "amenity=drinking_water", FOUNTAIN_DEDUP),
            (
                "berlin:layer",
                "ac_trinkwasserbrunnen_wasserspender",
                FOUNTAIN_DEDUP,
            ),
        ],
    },
    // The full ALKIS fountains and wells (311) beat the OSM ones. OSM has no monument rule, so a
    // NAS 526 has nothing to beat.
    PointDedup {
        winner_code: Some("311"),
        winner: ("freemapper:source", "alkis-nas/*"),
        losers: &[
            ("freemapper:osm", "amenity=fountain", FOUNTAIN_DEDUP),
            ("freemapper:osm", "man_made=water_well", FOUNTAIN_DEDUP),
        ],
    },
    // The surveyed fountains, pumps and drinking fountains beat the OSM ones (ADR 0087). The
    // fountain registers and the full ALKIS above have gone first, so a survey point they
    // dropped wins nothing.
    PointDedup {
        winner_code: None,
        winner: ("berlin:layer", "af_wasserpumpen_brunnen"),
        losers: &[
            ("freemapper:osm", "man_made=water_well", FOUNTAIN_DEDUP),
            ("freemapper:osm", "amenity=fountain", FOUNTAIN_DEDUP),
        ],
    },
    PointDedup {
        winner_code: None,
        winner: ("berlin:layer", "bv_springbrunnen_zierbrunnen"),
        losers: &[
            ("freemapper:osm", "amenity=fountain", FOUNTAIN_DEDUP),
            ("freemapper:osm", "man_made=water_well", FOUNTAIN_DEDUP),
        ],
    },
    PointDedup {
        winner_code: None,
        winner: ("berlin:layer", "ac_trinkwasserbrunnen_wasserspender"),
        losers: &[("freemapper:osm", "amenity=drinking_water", FOUNTAIN_DEDUP)],
    },
    // The cadastre monuments (526) beat the OSM memorials beside them (ADR 0075, 0078).
    PointDedup {
        winner_code: Some("526"),
        winner: ("freemapper:source", "alkis-nas/*"),
        losers: MEMORIAL_LOSERS,
    },
    PointDedup {
        winner_code: Some("526"),
        winner: ("freemapper:source", "geoportal-berlin/bauwerke-flaechen"),
        losers: MEMORIAL_LOSERS,
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
    // A surveyed bollard beats an OSM one (a control site on sprint maps, ADR 0077).
    PointDedup {
        winner_code: None,
        winner: ("berlin:layer", "av_poller"),
        losers: &[("freemapper:osm", "barrier=bollard", FURNITURE_DEDUP_M)],
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

/// The OSM memorial kinds of the 526 rule. A cadastre monument is an area, its point the centre
/// of the area, and the OSM node can stand anywhere on it: 5 m is 1.25 mm on paper at 1:4000.
const MEMORIAL_DEDUP_M: Limit = Limit::Metres(5.0);
const MEMORIAL_LOSERS: &[(&str, &str, Limit)] = &[
    ("freemapper:osm", "historic=memorial", MEMORIAL_DEDUP_M),
    ("freemapper:osm", "historic=monument", MEMORIAL_DEDUP_M),
    ("freemapper:osm", "historic=wayside_cross", MEMORIAL_DEDUP_M),
    (
        "freemapper:osm",
        "historic=boundary_stone",
        MEMORIAL_DEDUP_M,
    ),
];

/// A survey mast or flag pole and an OSM one at the same place are one pole, whichever word
/// each source uses.
const SURVEY_MAST_LOSERS: &[(&str, &str, Limit)] = &[
    ("freemapper:osm", "man_made=mast", FURNITURE_DEDUP_M),
    ("freemapper:osm", "man_made=flagpole", FURNITURE_DEDUP_M),
];

/// Whether the tag value `value` holds `wanted`: one of its `; `-separated parts equals it, or
/// starts with it when `wanted` ends in `*`.
pub(crate) fn tag_holds(value: &str, wanted: &str) -> bool {
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
/// water) within [`FOUNTAIN_DEDUP`] of a point of the fountain (drinking water) register, an
/// OSM fountain or well within [`FOUNTAIN_DEDUP`] of a full ALKIS (NAS) one, an OSM fountain,
/// well or drinking-water point within [`FOUNTAIN_DEDUP`] of the survey one (ADR 0087), and
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
            let Some(limit) = rule
                .losers
                .iter()
                .find(|&&(key, value, _)| holds(object, (key, value)))
                .map(|&(_, _, limit)| limit)
            else {
                return;
            };
            let Some((x, y)) = position(object).filter(|_| !*gone) else {
                return;
            };
            let limit = limit.map_units(scale_denominator);
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
        MapKind::Schoolyard.symbol_set(1000)
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

    const SURVEY: &str = "geoportal-berlin/sb-objekte";
    const WELL: (&str, &str) = ("freemapper:osm", "man_made=water_well");

    fn survey_pump(id: &str, de: f64) -> MapObject {
        point(
            "311",
            SURVEY,
            id,
            de,
            &[("berlin:layer", "af_wasserpumpen_brunnen")],
        )
    }

    fn osm_well(id: &str, de: f64) -> MapObject {
        point("311", "osm", id, de, &[WELL])
    }

    /// The pairs measured in the Viktoriapark (Berlin, 2026-11-10): the surveyed street pump and
    /// the OSM well lie 0.05, 0.44 and 0.49 m apart; the fourth pair is 9.77 m apart.
    #[test]
    fn survey_pump_beats_the_osm_well_at_the_measured_distances() {
        let cadastre = vec![
            survey_pump("Pk_25.09.2014_3", 0.0),
            survey_pump("Pk_29.04.2015_4", 100.0),
            survey_pump("Pk_29.04.2015_6", 200.0),
            survey_pump("Pk_29.04.2015_7", 300.0),
        ];
        let osm = vec![
            osm_well("node/1313558844", 0.05),
            osm_well("node/6005240121", 100.44),
            osm_well("node/1154543493", 200.49),
            osm_well("node/966049521", 309.77),
        ];
        let (osm, cadastre) = drop_points_near_registers(osm, cadastre, &set(), SCALE);
        assert_eq!(ids(&osm), ["node/966049521"], "9.77 m is another object");
        assert_eq!(ids(&cadastre).len(), 4);
        assert_eq!(
            tag(&cadastre[0], "freemapper:also"),
            Some("osm/node/1313558844")
        );
        assert_eq!(
            tag(&cadastre[2], "freemapper:also"),
            Some("osm/node/1154543493")
        );
        assert_eq!(tag(&cadastre[3], "freemapper:also"), None);
    }

    #[test]
    fn register_beats_survey_beats_osm_for_one_fountain() {
        let register = "geoportal-berlin/zierbrunnen";
        let cadastre = vec![
            point("311", register, "TB-FK-5", 0.0, &[]),
            survey_pump("Pk_1", 1.0),
            // Beside the survey pump, 3.2 m from the register point.
            survey_pump("Pk_2", 50.0),
        ];
        let osm = vec![
            osm_well("node/1", 2.0),
            osm_well("node/2", 50.5),
            osm_well("node/3", 3.2),
        ];
        let (osm, cadastre) = drop_points_near_registers(osm, cadastre, &set(), SCALE);
        // node/1 goes to the register, node/2 to the second pump; node/3 is 3.2 m from the
        // register and the first pump is gone: it stays (0.8 mm on paper).
        assert_eq!(ids(&osm), ["node/3"]);
        assert_eq!(ids(&cadastre), ["TB-FK-5", "Pk_2"]);
        let also = tag(&cadastre[0], "freemapper:also").unwrap();
        assert!(also.contains("geoportal-berlin/sb-objekte/Pk_1"), "{also}");
        assert!(also.contains("osm/node/1"), "{also}");
        assert_eq!(tag(&cadastre[1], "freemapper:also"), Some("osm/node/2"));
    }

    #[test]
    fn fountains_close_but_different_stay() {
        // Two OSM wells 3 m apart without a winner, and a pump with a well just over the limit
        // (0.8 mm on paper).
        let osm = vec![
            osm_well("node/1", 0.0),
            osm_well("node/2", 3.0),
            osm_well("node/3", 103.2),
        ];
        let cadastre = vec![survey_pump("Pk_1", 100.0)];
        let (osm, cadastre) = drop_points_near_registers(osm, cadastre, &set(), SCALE);
        assert_eq!(ids(&osm), ["node/1", "node/2", "node/3"]);
        assert_eq!(ids(&cadastre), ["Pk_1"]);
        assert_eq!(tag(&cadastre[0], "freemapper:also"), None);
    }

    /// The limit is 0.75 mm on paper, and 3 m on the ground at least. The points stand 0.625
    /// and 0.875 mm apart on paper.
    #[test]
    fn the_fountain_limit_is_paper_based_with_a_ground_floor() {
        for (scale, dropped) in [
            (1000, ["near", "far"].as_slice()),
            (4000, &["near"]),
            (5000, &["near"]),
        ] {
            let cadastre = vec![survey_pump("Pk_1", 0.0)];
            let osm = vec![osm_well("near", 2.5), osm_well("far", 3.5)];
            let (osm, _) = drop_points_near_registers(osm, cadastre, &set(), scale);
            let kept: Vec<&str> = ["near", "far"]
                .into_iter()
                .filter(|id| !dropped.contains(id))
                .collect();
            assert_eq!(ids(&osm), kept, "1:{scale}");
        }
    }

    #[test]
    fn survey_tap_beats_the_osm_drinking_water() {
        let cadastre = vec![point(
            "313",
            SURVEY,
            "Pk_9",
            0.0,
            &[("berlin:layer", "ac_trinkwasserbrunnen_wasserspender")],
        )];
        let drinking = |id: &str, de: f64| {
            point(
                "313",
                "osm",
                id,
                de,
                &[("freemapper:osm", "amenity=drinking_water")],
            )
        };
        let osm = vec![drinking("node/1", 0.4), drinking("node/2", 6.0)];
        let (osm, cadastre) = drop_points_near_registers(osm, cadastre, &set(), SCALE);
        assert_eq!(ids(&osm), ["node/2"]);
        assert_eq!(tag(&cadastre[0], "freemapper:also"), Some("osm/node/1"));
    }

    #[test]
    fn register_fountain_beats_the_osm_well() {
        let cadastre = vec![point(
            "311",
            "geoportal-berlin/zierbrunnen",
            "TB-FK-5",
            0.0,
            &[],
        )];
        let osm = vec![osm_well("node/1", 1.0)];
        let (osm, cadastre) = drop_points_near_registers(osm, cadastre, &set(), SCALE);
        assert!(osm.is_empty());
        assert_eq!(tag(&cadastre[0], "freemapper:also"), Some("osm/node/1"));
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

    fn sprint() -> SymbolSet {
        MapKind::Sprint.symbol_set(4000)
    }

    /// A closed rectangle `w` by `h` metres with its corner at (`e`, `n`) metres, from `source`.
    fn area(
        code: &str,
        source: &str,
        id: &str,
        (e, n): (f64, f64),
        (w, h): (f64, f64),
        extra: &[(&str, &str)],
    ) -> MapObject {
        let at = |de: f64, dn: f64| {
            MapCoord::new(
                ((e + de) * f64::from(M)) as i32,
                ((n + dn) * f64::from(M)) as i32,
            )
        };
        let mut tags = vec![("freemapper:source", source), ("freemapper:source_id", id)];
        tags.extend_from_slice(extra);
        object(
            code,
            MapObjectKind::Path,
            vec![at(0.0, 0.0), at(w, 0.0), at(w, h), at(0.0, h), at(0.0, 0.0)],
            &tags,
        )
    }

    #[test]
    fn osm_passages_over_cadastre_passages_and_canopies_are_dropped() {
        let set = sprint();
        let osm_tags = [("freemapper:osm", "tunnel=building_passage")];
        let candidates = vec![
            // The Berlin building part and an OSM strip over most of it.
            area(
                "522",
                "geoportal-berlin/building-parts",
                "BP1",
                (0.0, 0.0),
                (12.0, 4.0),
                &[],
            ),
            area("522", "osm", "way/1", (0.0, 0.5), (12.0, 3.0), &osm_tags),
            // An OSM strip beside it, sharing a quarter of its length: another passage.
            area("522", "osm", "way/2", (9.0, 0.5), (12.0, 3.0), &osm_tags),
            // An OSM strip over an ALKIS canopy of the map.
            area("522", "osm", "way/3", (100.0, 0.0), (10.0, 3.0), &osm_tags),
            // An OSM strip over another survey object that ranks above it, the survey shelter.
            area(
                "522",
                "geoportal-berlin/sb-objekte",
                "S1",
                (200.0, 0.0),
                (6.0, 3.0),
                &[("berlin:layer", "br_fahrgastunterstand")],
            ),
            area("522", "osm", "way/4", (200.0, 0.0), (6.0, 3.0), &osm_tags),
            // A survey shelter over a building part: the building part ranks higher.
            area(
                "522",
                "geoportal-berlin/building-parts",
                "BP2",
                (300.0, 0.0),
                (6.0, 3.0),
                &[],
            ),
            area(
                "522",
                "geoportal-berlin/sb-objekte",
                "S2",
                (300.0, 0.0),
                (6.0, 3.0),
                &[],
            ),
            // The same rectangle in another symbol is no duplicate.
            area("520", "osm", "way/5", (0.0, 0.0), (12.0, 4.0), &[]),
        ];
        let mut canopies = vec![area(
            "522",
            "alkis-nas/ueberdachung",
            "C1",
            (99.5, -0.5),
            (11.0, 4.0),
            &[],
        )];
        let kept = drop_areas_by_rank(candidates, &mut [&mut canopies], &set);
        assert_eq!(ids(&kept), ["BP1", "way/2", "S1", "BP2", "way/5"]);
        assert_eq!(tag(&kept[0], "freemapper:also"), Some("osm/way/1"));
        assert_eq!(tag(&canopies[0], "freemapper:also"), Some("osm/way/3"));
        assert_eq!(tag(&kept[2], "freemapper:also"), Some("osm/way/4"));
        assert_eq!(
            tag(&kept[3], "freemapper:also"),
            Some("geoportal-berlin/sb-objekte/S2")
        );
    }

    #[test]
    fn a_dropped_area_drops_nothing_else() {
        // Chain A > B > C (building part, survey shelter, OSM strip). B lies mostly under A and
        // goes; C lies mostly under B but only 44 % under A, so C stays.
        let set = sprint();
        let osm_tags = [("freemapper:osm", "tunnel=building_passage")];
        let a = area(
            "522",
            "geoportal-berlin/building-parts",
            "A",
            (0.0, 0.0),
            (6.0, 3.0),
            &[],
        );
        let b = area(
            "522",
            "geoportal-berlin/sb-objekte",
            "B",
            (0.0, 0.0),
            (12.0, 3.0),
            &[],
        );
        let c = area("522", "osm", "way/C", (0.0, 0.0), (16.0, 3.0), &osm_tags);
        let kept = drop_areas_by_rank(vec![c, b, a], &mut [], &set);
        assert_eq!(ids(&kept), ["way/C", "A"]);
        assert_eq!(
            tag(&kept[1], "freemapper:also"),
            Some("geoportal-berlin/sb-objekte/B")
        );
        assert_eq!(tag(&kept[0], "freemapper:also"), None);
    }

    #[test]
    fn the_note_goes_to_the_surviving_winner() {
        // C lies under A (58 %) and under B; B goes because of A, so A, not B, gets the note.
        let set = sprint();
        let osm_tags = [("freemapper:osm", "tunnel=building_passage")];
        let a = area(
            "522",
            "geoportal-berlin/building-parts",
            "A",
            (0.0, 0.0),
            (6.0, 3.0),
            &[],
        );
        let b = area(
            "522",
            "geoportal-berlin/sb-objekte",
            "B",
            (0.0, 0.0),
            (12.0, 3.0),
            &[],
        );
        let c = area("522", "osm", "way/C", (0.0, 0.0), (12.0, 3.0), &osm_tags);
        let kept = drop_areas_by_rank(vec![a, b, c], &mut [], &set);
        assert_eq!(ids(&kept), ["A"]);
        assert_eq!(
            tag(&kept[0], "freemapper:also"),
            Some("geoportal-berlin/sb-objekte/B; osm/way/C")
        );
    }

    #[test]
    fn the_overlap_has_a_tolerance_in_paper_millimetres() {
        // The winner lies `gap` metres inside the OSM strip on every side. The strip is 20 by
        // 8 m. Without a tolerance the winner covers 53 % at a gap of 1.5 m and 35 % at 2.2 m;
        // the tolerance of 0.25 mm on paper (1 m at 1:4000) widens the winner by 1 m.
        let set = sprint();
        let osm_tags = [("freemapper:osm", "tunnel=building_passage")];
        let kept = |gap: f64| {
            let candidates = vec![
                area(
                    "522",
                    "alkis",
                    "A",
                    (gap, gap),
                    (20.0 - 2.0 * gap, 8.0 - 2.0 * gap),
                    &[],
                ),
                area("522", "osm", "way/1", (0.0, 0.0), (20.0, 8.0), &osm_tags),
            ];
            drop_areas_by_rank(candidates, &mut [], &set).len()
        };
        assert_eq!(kept(1.5), 1);
        assert_eq!(kept(2.2), 1, "35 % plus the tolerance is 62 %");
        assert_eq!(kept(3.5), 2, "13 m2 plus the tolerance is 28 %");
    }

    #[test]
    fn osm_private_strips_inside_alkis_out_of_bounds_areas_are_dropped() {
        let set = sprint();
        let osm = [("freemapper:osm", "highway=service; access=private")];
        let candidates = vec![
            area("520", "osm", "way/1", (0.0, 0.0), (20.0, 3.0), &osm),
            // Only 60 % inside: the rest is news.
            area("520", "osm", "way/2", (10.0, 10.0), (20.0, 3.0), &osm),
            area("520", "osm", "way/3", (500.0, 0.0), (20.0, 3.0), &osm),
        ];
        // way/2 sticks out to the right of the land use area.
        let mut out_of_bounds = vec![area(
            "520",
            "alkis-nutzung",
            "N1",
            (-5.0, -5.0),
            (22.0, 25.0),
            &[],
        )];
        let kept = drop_areas_by_rank(candidates, &mut [&mut out_of_bounds], &set);
        assert_eq!(ids(&kept), ["way/2", "way/3"]);
        assert_eq!(tag(&out_of_bounds[0], "freemapper:also"), Some("osm/way/1"));
    }

    #[test]
    fn osm_bridges_over_berlin_bridges_are_dropped_but_paved_areas_stay() {
        let set = sprint();
        let candidates = vec![
            area(
                "501.5",
                "geoportal-berlin/bauwerke-flaechen",
                "B1",
                (0.0, 0.0),
                (30.0, 10.0),
                &[("berlin:layer", "bauwerke-flaechen")],
            ),
            area(
                "501.5",
                "osm",
                "way/1",
                (0.0, 0.0),
                (30.0, 10.0),
                &[("freemapper:osm", "man_made=bridge")],
            ),
            // A surveyed pavement is not a bridge: it is no winner for an OSM bridge.
            area(
                "501.5",
                "geoportal-berlin/sb-flaechen",
                "F1",
                (100.0, 0.0),
                (30.0, 10.0),
                &[("berlin:layer", "cl_gehweg")],
            ),
            area(
                "501.5",
                "osm",
                "way/2",
                (100.0, 0.0),
                (30.0, 10.0),
                &[("freemapper:osm", "man_made=bridge")],
            ),
            // An OSM pavement over a bridge stays too: only OSM bridges are matched.
            area(
                "501.5",
                "osm",
                "way/3",
                (0.0, 0.0),
                (30.0, 10.0),
                &[("freemapper:osm", "highway=pedestrian")],
            ),
        ];
        let kept = drop_areas_by_rank(candidates, &mut [], &set);
        assert_eq!(ids(&kept), ["B1", "F1", "way/2", "way/3"]);
    }

    #[test]
    fn the_land_use_candidates_rank_as_cadastre() {
        assert_eq!(
            rank(&point("520", "alkis-nutzung", "1", 0.0, &[])),
            Some(Rank::Alkis)
        );
    }

    #[test]
    fn osm_memorials_beside_cadastre_monuments_are_dropped() {
        let set = sprint();
        let cadastre = vec![
            point(
                "526",
                "geoportal-berlin/bauwerke-flaechen",
                "D1",
                0.0,
                &[("berlin:layer", "bauwerke-flaechen")],
            ),
            point("526", "alkis-nas/sonstiges-bauwerk", "N1", 100.0, &[]),
        ];
        let osm = vec![
            point(
                "526",
                "osm",
                "node/1",
                3.0,
                &[("freemapper:osm", "historic=memorial")],
            ),
            point(
                "526",
                "osm",
                "node/2",
                104.0,
                &[("freemapper:osm", "historic=monument")],
            ),
            // 8 m away: another memorial.
            point(
                "526",
                "osm",
                "node/3",
                8.0,
                &[("freemapper:osm", "historic=memorial")],
            ),
        ];
        let (osm, cadastre) = drop_points_near_registers(osm, cadastre, &set, SCALE);
        assert_eq!(ids(&osm), ["node/3"]);
        assert_eq!(tag(&cadastre[0], "freemapper:also"), Some("osm/node/1"));
        assert_eq!(tag(&cadastre[1], "freemapper:also"), Some("osm/node/2"));
    }

    #[test]
    fn osm_kerbs_and_embankments_along_cadastre_ones_are_dropped() {
        let set = sprint();
        let mut cadastre = vec![
            line("501.1", "geoportal-berlin/sb-bordstein", "K1", 0.0, &[]),
            line("104", "geoportal-berlin/relief", "R1", 10.0, &[]),
        ];
        let osm = vec![
            line(
                "501.1",
                "osm",
                "way/1",
                0.5,
                &[("freemapper:osm", "barrier=kerb")],
            ),
            line(
                "104",
                "osm",
                "way/2",
                10.5,
                &[("freemapper:osm", "man_made=embankment")],
            ),
        ];
        let kept = drop_osm_lines_near_cadastre(osm, &mut cadastre, &set, SCALE);
        assert!(kept.is_empty());
    }
}
