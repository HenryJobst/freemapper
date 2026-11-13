//! Walls, fences and hedges that overlap on paper (ISSprOM 2019-2 §3.3, §521; ADR 0068).
//!
//! The full ALKIS, its paired wall edges, OSM and the Berlin survey each give lines for the same
//! wall, fence or hedge, often with different symbols, and some walls run along a building's
//! outline. The same-symbol duplicates are gone before (`build/dedup.rs`); here a line loses the
//! sections that run parallel to a line it yields to, closer than the two symbols may come, and
//! the sections along a building outline. A tree standing in a hedge is part of the hedge, and a
//! tree on a wall or fence line gives way to it.

use std::cmp::Ordering;
use std::collections::HashMap;

use geo_types::Polygon;

use crate::build::dedup::{Rank, add_also, rank};
use crate::build::geometry::{distance_to_segment, xy};
use crate::build::symbol_measures::{FENCE_MM, HEDGE_MM, LINE_GAP_MM, RETAINED_WALL_MM, WALL_MM};
use crate::crs::ProjectedPoint;
use crate::georef::{GeorefError, Georeferencing};
use crate::measure::{SymbolMm, SymbolScale, map_units_per_metre};
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet};
use crate::project::MapKind;

/// The line symbols of walls, fences and hedges the build emits, the strongest first, with their
/// width on paper at 1:4000 (mm) and whether they are black. Of two lines for one object the
/// stronger symbol wins at equal rank: the uncrossable wall (515, min. 0.4 mm), the uncrossable
/// fence (518, 0.4 mm), the hedge (410.1, green, 0.4 mm), the passable retained wall (513.2, line
/// 0.21 mm with half dots of 0.6 mm). A solid construction beats a fence, a fence beats
/// vegetation, and what must not be crossed beats what may be crossed: a runner never gets an
/// opening that a more trusted line closes.
pub const BARRIER_SYMBOLS: [(&str, SymbolMm, bool); 4] = [
    ("515", WALL_MM, true),
    ("518", FENCE_MM, true),
    ("410.1", HEDGE_MM, false),
    ("513.2", RETAINED_WALL_MM, true),
];
/// Lines of two sources this close (ground metres) are one object whatever their symbols: the
/// tolerance of the same-symbol duplicates (`build/dedup.rs`).
pub const SOURCE_TOLERANCE_M: f64 = 1.5;
/// A line this close (ground metres) to a building outline and parallel to it is the building's
/// wall: the outline already draws it.
pub const BUILDING_EDGE_M: f64 = 0.5;
/// Two sections run parallel when they are at most 30° apart (the cosine).
const PARALLEL_COS: f64 = 0.866;
/// A line is walked in pieces of at most this length (ground metres).
const STEP_M: f64 = 0.25;
/// A piece left over after the cut must be at least this long on paper (mm): the norm's opening
/// of fences and walls (§3.3, 1.0 mm), so no stub shorter than a gap remains.
pub const PIECE_MIN_MM: SymbolMm = SymbolMm(1.0);
/// A tree whose centre lies this close (ground metres) to a hedge (410.1) is part of the hedge;
/// on a wall or fence its symbol covers the line (at 1:4000 1 m is 0.25 mm, less than half the
/// 0.8 mm tree symbol), and the uncrossable line matters more to a runner.
pub const TREE_ON_LINE_M: f64 = 1.0;
/// The tree symbols.
const TREE_CODES: [&str; 2] = ["417", "418"];
/// The `freemapper:generalized` value of a line cut along a building outline.
const ALONG_BUILDING: &str = "cut along a building (521)";

type P = (f64, f64);

fn tag<'a>(object: &'a MapObject, key: &str) -> Option<&'a str> {
    object
        .tags
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

/// `<source>/<source_id>` of an object.
fn name(object: &MapObject) -> String {
    format!(
        "{}/{}",
        tag(object, "freemapper:source").unwrap_or_default(),
        tag(object, "freemapper:source_id").unwrap_or_default()
    )
}

/// Whether the directions `u` and `v` are at most 30° apart, either way round.
fn parallel(u: P, v: P) -> bool {
    let (lu, lv) = (u.0.hypot(u.1), v.0.hypot(v.1));
    lu > 0.0 && lv > 0.0 && ((u.0 * v.0 + u.1 * v.1) / (lu * lv)).abs() >= PARALLEL_COS
}

/// Segments in a grid of `cell`-sized cells along each segment.
struct Segments {
    cell: f64,
    /// (owner, a, b): the owner is an index into the kept lines, `None` for a building.
    items: Vec<(Option<usize>, P, P)>,
    grid: HashMap<(i64, i64), Vec<usize>>,
}

impl Segments {
    fn new(cell: f64) -> Self {
        Self {
            cell,
            items: Vec::new(),
            grid: HashMap::new(),
        }
    }

    fn key(&self, p: P) -> (i64, i64) {
        (
            (p.0 / self.cell).floor() as i64,
            (p.1 / self.cell).floor() as i64,
        )
    }

    fn add(&mut self, owner: Option<usize>, a: P, b: P) {
        let id = self.items.len();
        self.items.push((owner, a, b));
        let steps = ((b.0 - a.0).hypot(b.1 - a.1) / (self.cell / 2.0))
            .ceil()
            .max(1.0) as usize;
        let mut keys: Vec<(i64, i64)> = (0..=steps)
            .map(|i| {
                let t = i as f64 / steps as f64;
                self.key((a.0 + t * (b.0 - a.0), a.1 + t * (b.1 - a.1)))
            })
            .collect();
        keys.sort_unstable();
        keys.dedup();
        for k in keys {
            self.grid.entry(k).or_default().push(id);
        }
    }

    /// The segments with a sample cell within two cells of `p`'s: every segment within one cell
    /// of `p`.
    fn near(&self, p: P) -> Vec<usize> {
        let (cx, cy) = self.key(p);
        let mut found: Vec<usize> = (cx - 2..=cx + 2)
            .flat_map(|x| (cy - 2..=cy + 2).map(move |y| (x, y)))
            .flat_map(|k| self.grid.get(&k).into_iter().flatten().copied())
            .collect();
        found.sort_unstable();
        found.dedup();
        found
    }
}

/// A wall, fence or hedge line taking part.
struct Barrier {
    /// 0: an OSM candidate, 1: a cadastre candidate.
    list: usize,
    index: usize,
    rank: Option<Rank>,
    /// Position in [`BARRIER_SYMBOLS`], 0 the strongest.
    strength: usize,
    width_mm: SymbolMm,
    black: bool,
    length: f64,
    name: String,
}

/// Cuts the overlapping walls, fences and hedges among the `osm` and `cadastre` candidates
/// (ADR 0068), the lines of [`BARRIER_SYMBOLS`]:
/// 1. the lines are taken strongest first: higher rank (register > ALKIS > survey > OSM), then
///    the stronger symbol, the longer line, the name;
/// 2. a line loses each section that runs parallel (at most 30° off) to a line taken before
///    it, closer than the two symbols may come on paper (half their widths, plus the 0.15 mm gap
///    of §3.3 for two black lines); for lines of two ranks at least [`SOURCE_TOLERANCE_M`], since that is one object
///    seen by two sources. Two lines of one rank with one symbol are left alone: they are two
///    objects (the edges of one wall are paired before, `build/walls.rs`);
/// 3. a line loses each section parallel to a building outline (`footprints`, the 521 map
///    objects in the map's CRS) within [`BUILDING_EDGE_M`];
/// 4. the pieces left must be [`PIECE_MIN_MM`] long; they keep the line's tags, a cut along a
///    building is noted in `freemapper:generalized`, and the winner names the cut line in
///    `freemapper:also`;
/// 5. a tree point (417, 418) within [`TREE_ON_LINE_M`] of one of these lines left after the
///    cut is dropped, and the nearest line names it in `freemapper:generalized`.
///
/// Everything else stays in its order; a cut line is replaced by its pieces in its place.
pub fn resolve_barrier_overlaps(
    osm: Vec<MapObject>,
    cadastre: Vec<MapObject>,
    footprints: &[Polygon<f64>],
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<(Vec<MapObject>, Vec<MapObject>), GeorefError> {
    let mut lists = [osm, cadastre];
    let symbols: Vec<(u32, usize, SymbolMm, bool)> = BARRIER_SYMBOLS
        .iter()
        .enumerate()
        .filter_map(|(strength, &(code, width, black))| {
            Some((symbol_set.id_for_code(code)?, strength, width, black))
        })
        .collect();
    let metre = map_units_per_metre(georef.scale_denominator);
    let scale = SymbolScale::new(kind, georef.scale_denominator);
    let length = |o: &MapObject| -> f64 {
        o.coords
            .windows(2)
            .map(|w| {
                let (a, b) = (xy(&w[0]), xy(&w[1]));
                (b.0 - a.0).hypot(b.1 - a.1)
            })
            .sum()
    };
    let mut barriers: Vec<Barrier> = Vec::new();
    for (list, objects) in lists.iter().enumerate() {
        for (index, o) in objects.iter().enumerate() {
            if !matches!(o.kind, MapObjectKind::Path) || o.coords.len() < 2 {
                continue;
            }
            if let Some(&(_, strength, width_mm, black)) =
                symbols.iter().find(|(id, _, _, _)| *id == o.symbol_id)
            {
                barriers.push(Barrier {
                    list,
                    index,
                    rank: rank(o),
                    strength,
                    width_mm,
                    black,
                    length: length(o),
                    name: name(o),
                });
            }
        }
    }
    barriers.sort_by(|a, b| {
        b.rank
            .cmp(&a.rank)
            .then(a.strength.cmp(&b.strength))
            .then(b.length.total_cmp(&a.length))
            .then(a.name.cmp(&b.name))
            .then(a.list.cmp(&b.list))
            .then(a.index.cmp(&b.index))
    });

    // The widest reach of a pair: two ranks, the widest symbols.
    let widest = SymbolMm(BARRIER_SYMBOLS.iter().map(|s| s.1.0).fold(0.0, f64::max));
    let reach = (LINE_GAP_MM + widest)
        .map_units(scale)
        .max(SOURCE_TOLERANCE_M * metre);
    let mut segments = Segments::new(reach.max(BUILDING_EDGE_M * metre));
    for footprint in footprints {
        for ring in std::iter::once(footprint.exterior()).chain(footprint.interiors()) {
            let points = ring
                .coords()
                .map(|c| {
                    georef
                        .projected_to_map(ProjectedPoint {
                            easting: c.x,
                            northing: c.y,
                        })
                        .map(|m| xy(&m))
                })
                .collect::<Result<Vec<P>, _>>()?;
            for w in points.windows(2) {
                segments.add(None, w[0], w[1]);
            }
        }
    }

    let step = STEP_M * metre;
    let piece_min = PIECE_MIN_MM.map_units(scale);
    let building = BUILDING_EDGE_M * metre;
    // The kept pieces of each line, by its position in `barriers`.
    let mut pieces: Vec<Option<Vec<Vec<MapCoord>>>> = (0..barriers.len()).map(|_| None).collect();
    // The names each kept line wins over, and whether a line was cut along a building.
    let mut also: Vec<Vec<String>> = vec![Vec::new(); barriers.len()];
    let mut along_building = vec![false; barriers.len()];
    for k in 0..barriers.len() {
        let b = &barriers[k];
        let object = &lists[b.list][b.index];
        let limit = |other: &Barrier| -> Option<f64> {
            // Lines of one colour keep the gap of §3.3; lines of two colours must not overlap.
            let gap = if b.black == other.black {
                LINE_GAP_MM
            } else {
                SymbolMm(0.0)
            };
            let gap = (gap + (b.width_mm + other.width_mm) / 2.0).map_units(scale);
            if other.rank != b.rank {
                Some(gap.max(SOURCE_TOLERANCE_M * metre))
            } else if other.strength != b.strength {
                Some(gap)
            } else {
                None
            }
        };
        // Walk the line in short pieces; a piece is covered when its middle lies close to a
        // parallel kept line or building edge.
        let mut runs: Vec<Vec<P>> = Vec::new();
        let mut current: Vec<P> = Vec::new();
        let mut cut = false;
        for w in object.coords.windows(2) {
            let (a, c) = (xy(&w[0]), xy(&w[1]));
            let length = (c.0 - a.0).hypot(c.1 - a.1);
            let n = (length / step).ceil().max(1.0) as usize;
            let direction = (c.0 - a.0, c.1 - a.1);
            for i in 0..n {
                let (t0, t1) = (i as f64 / n as f64, (i + 1) as f64 / n as f64);
                let at = |t: f64| (a.0 + t * direction.0, a.1 + t * direction.1);
                let middle = at((t0 + t1) / 2.0);
                let mut covered = false;
                for id in segments.near(middle) {
                    let (owner, s0, s1) = segments.items[id];
                    let reach = match owner {
                        None => building,
                        Some(j) => match limit(&barriers[j]) {
                            Some(l) => l,
                            None => continue,
                        },
                    };
                    if distance_to_segment(middle, s0, s1) < reach
                        && parallel(direction, (s1.0 - s0.0, s1.1 - s0.1))
                    {
                        covered = true;
                        match owner {
                            None => along_building[k] = true,
                            Some(j) => {
                                if !also[j].contains(&b.name) {
                                    also[j].push(b.name.clone());
                                }
                            }
                        }
                        break;
                    }
                }
                if covered {
                    cut = true;
                    if current.len() > 1 {
                        runs.push(std::mem::take(&mut current));
                    }
                    current.clear();
                } else {
                    if current.is_empty() {
                        current.push(at(t0));
                    }
                    current.push(at(t1));
                }
            }
        }
        if current.len() > 1 {
            runs.push(current);
        }
        let kept: Vec<Vec<MapCoord>> = if cut {
            runs.into_iter()
                .filter(|run| {
                    run.windows(2)
                        .map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1))
                        .sum::<f64>()
                        >= piece_min
                })
                .map(|run| simplify_run(&run))
                .collect()
        } else {
            vec![object.coords.clone()]
        };
        for run in &kept {
            for w in run.windows(2) {
                segments.add(Some(k), xy(&w[0]), xy(&w[1]));
            }
        }
        pieces[k] = Some(kept);
    }

    // The objects that replace each line, by list and index.
    let mut replaced: [HashMap<usize, Vec<MapObject>>; 2] = [HashMap::new(), HashMap::new()];
    for (k, b) in barriers.iter().enumerate() {
        let object = &lists[b.list][b.index];
        let mut tags = object.tags.clone();
        let mut names = std::mem::take(&mut also[k]);
        names.sort_unstable();
        for n in names {
            add_also(&mut tags, &n);
        }
        if along_building[k] {
            tags.push((
                "freemapper:generalized".to_string(),
                ALONG_BUILDING.to_string(),
            ));
        }
        let objects = pieces[k]
            .take()
            .unwrap_or_default()
            .into_iter()
            .map(|coords| MapObject {
                symbol_id: object.symbol_id,
                kind: MapObjectKind::Path,
                coords,
                tags: tags.clone(),
            })
            .collect();
        replaced[b.list].insert(b.index, objects);
    }
    for (list, objects) in lists.iter_mut().enumerate() {
        let old = std::mem::take(objects);
        for (index, object) in old.into_iter().enumerate() {
            match replaced[list].remove(&index) {
                Some(new) => objects.extend(new),
                None => objects.push(object),
            }
        }
    }
    let [osm, cadastre] = lists;
    let line_ids: Vec<u32> = symbols.iter().map(|s| s.0).collect();
    Ok(drop_trees_on_lines(
        osm, cadastre, &line_ids, symbol_set, metre,
    ))
}

/// The run's points without the walking steps on straight stretches, as map coordinates.
fn simplify_run(run: &[P]) -> Vec<MapCoord> {
    let mut kept: Vec<P> = Vec::with_capacity(run.len());
    for &p in run {
        if kept.len() >= 2 {
            let (a, b) = (kept[kept.len() - 2], kept[kept.len() - 1]);
            let (u, v) = ((b.0 - a.0, b.1 - a.1), (p.0 - b.0, p.1 - b.1));
            // Collinear within a micrometre of map coordinates: the step point goes.
            let cross = u.0 * v.1 - u.1 * v.0;
            let lu = u.0.hypot(u.1);
            if lu > 0.0 && (cross / lu).abs() < 1.0 && u.0 * v.0 + u.1 * v.1 > 0.0 {
                kept.pop();
            }
        }
        kept.push(p);
    }
    kept.into_iter()
        .map(|(x, y)| MapCoord::new(x.round() as i32, y.round() as i32))
        .collect()
}

/// Drops the tree points within [`TREE_ON_LINE_M`] of a line with one of the `line_ids` among
/// `osm` and `cadastre`; the nearest line names them in `freemapper:generalized`.
fn drop_trees_on_lines(
    osm: Vec<MapObject>,
    cadastre: Vec<MapObject>,
    line_ids: &[u32],
    symbol_set: &SymbolSet,
    metre: f64,
) -> (Vec<MapObject>, Vec<MapObject>) {
    let tree_ids: Vec<u32> = TREE_CODES
        .iter()
        .filter_map(|c| symbol_set.id_for_code(c))
        .collect();
    let limit = TREE_ON_LINE_M * metre;
    let mut lists = [osm, cadastre];
    let mut hedges = Segments::new(limit.max(1.0));
    let mut owners: Vec<(usize, usize)> = Vec::new();
    for (list, objects) in lists.iter().enumerate() {
        for (index, o) in objects.iter().enumerate() {
            if line_ids.contains(&o.symbol_id) && matches!(o.kind, MapObjectKind::Path) {
                let owner = owners.len();
                owners.push((list, index));
                for w in o.coords.windows(2) {
                    hedges.add(Some(owner), xy(&w[0]), xy(&w[1]));
                }
            }
        }
    }
    if owners.is_empty() {
        let [osm, cadastre] = lists;
        return (osm, cadastre);
    }
    let mut named: HashMap<usize, Vec<String>> = HashMap::new();
    let mut dropped = [vec![false; lists[0].len()], vec![false; lists[1].len()]];
    for (list, objects) in lists.iter().enumerate() {
        for (index, o) in objects.iter().enumerate() {
            let (MapObjectKind::Point { .. }, Some(c)) = (&o.kind, o.coords.first()) else {
                continue;
            };
            if !tree_ids.contains(&o.symbol_id) {
                continue;
            }
            let p = xy(c);
            let nearest = hedges
                .near(p)
                .into_iter()
                .map(|id| {
                    let (owner, a, b) = hedges.items[id];
                    (distance_to_segment(p, a, b), owner.unwrap_or_default())
                })
                .filter(|(d, _)| *d <= limit)
                .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
            if let Some((_, owner)) = nearest {
                dropped[list][index] = true;
                named.entry(owner).or_default().push(name(o));
            }
        }
    }
    for (owner, mut names) in named {
        let (list, index) = owners[owner];
        names.sort_unstable();
        names.dedup();
        lists[list][index]
            .tags
            .push(("freemapper:generalized".to_string(), names.join("; ")));
    }
    let [osm, cadastre] = lists;
    let keep = |objects: Vec<MapObject>, dropped: &[bool]| -> Vec<MapObject> {
        objects
            .into_iter()
            .zip(dropped)
            .filter(|(_, d)| !**d)
            .map(|(o, _)| o)
            .collect()
    };
    (keep(osm, &dropped[0]), keep(cadastre, &dropped[1]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::points::test_support::{Fixture, fixture};

    const ALKIS: &str = "alkis-nas/sonstiges-bauwerk";

    /// A line `code` from `source` through the points (ground metres from the reference point).
    fn line(f: &Fixture, code: &str, source: &str, id: &str, points: &[(f64, f64)]) -> MapObject {
        MapObject {
            symbol_id: f.id(code),
            kind: MapObjectKind::Path,
            coords: points.iter().map(|&p| f.at(p)).collect(),
            tags: vec![
                ("freemapper:source".to_string(), source.to_string()),
                ("freemapper:source_id".to_string(), id.to_string()),
            ],
        }
    }

    fn run(
        f: &Fixture,
        osm: Vec<MapObject>,
        cadastre: Vec<MapObject>,
        footprints: &[Polygon<f64>],
    ) -> (Vec<MapObject>, Vec<MapObject>) {
        resolve_barrier_overlaps(osm, cadastre, footprints, f.kind, &f.georef, &f.set).unwrap()
    }

    /// The length of a line in ground metres.
    fn metres(f: &Fixture, o: &MapObject) -> f64 {
        o.coords
            .windows(2)
            .map(|w| {
                let (a, b) = (
                    f.georef.map_to_projected(w[0]),
                    f.georef.map_to_projected(w[1]),
                );
                (b.easting - a.easting).hypot(b.northing - a.northing)
            })
            .sum()
    }

    #[test]
    fn osm_wall_along_an_alkis_fence_goes() {
        let f = fixture(MapKind::Sprint);
        // OSM barrier=wall (515) 1 m beside an ALKIS fence (518): one object, ALKIS wins.
        let osm = vec![line(&f, "515", "osm", "way/1", &[(0.0, 1.0), (40.0, 1.0)])];
        let fence = line(&f, "518", ALKIS, "f1", &[(0.0, 0.0), (40.0, 0.0)]);
        let (osm, cadastre) = run(&f, osm, vec![fence.clone()], &[]);
        assert!(osm.is_empty(), "{osm:?}");
        assert_eq!(cadastre.len(), 1);
        assert_eq!(cadastre[0].coords, fence.coords);
        assert_eq!(tag(&cadastre[0], "freemapper:also"), Some("osm/way/1"));
    }

    #[test]
    fn osm_retaining_wall_on_an_alkis_wall_goes() {
        let f = fixture(MapKind::Sprint);
        // Sanssouci: OSM barrier=retaining_wall (513.2) 0.7 m from the paired ALKIS wall (515).
        let osm = vec![line(
            &f,
            "513.2",
            "osm",
            "way/2",
            &[(0.0, 0.7), (30.0, 0.9)],
        )];
        let wall = line(&f, "515", ALKIS, "w1", &[(0.0, 0.0), (30.0, 0.0)]);
        let (osm, cadastre) = run(&f, osm, vec![wall], &[]);
        assert!(osm.is_empty(), "{osm:?}");
        assert_eq!(cadastre.len(), 1);
    }

    #[test]
    fn fence_on_a_wall_of_the_same_source_gives_way_to_the_wall() {
        let f = fixture(MapKind::Sprint);
        // An ALKIS fence 0.8 m beside an ALKIS wall: closer than 0.55 mm (2.2 m), the wall stays.
        let wall = line(&f, "515", ALKIS, "w1", &[(0.0, 0.0), (30.0, 0.0)]);
        let fence = line(&f, "518", ALKIS, "f1", &[(0.0, 0.8), (30.0, 0.8)]);
        let (_, cadastre) = run(&f, vec![], vec![fence, wall.clone()], &[]);
        assert_eq!(cadastre.len(), 1, "{cadastre:?}");
        assert_eq!(cadastre[0].coords, wall.coords);
        assert_eq!(
            tag(&cadastre[0], "freemapper:also"),
            Some(&*format!("{ALKIS}/f1"))
        );
        // A hedge 1 m beside a fence: the fence stays.
        let fence = line(&f, "518", ALKIS, "f1", &[(0.0, 0.0), (30.0, 0.0)]);
        let hedge = line(
            &f,
            "410.1",
            "alkis-nas/vegetationsmerkmal",
            "h1",
            &[(0.0, 1.0), (30.0, 1.0)],
        );
        let (_, cadastre) = run(&f, vec![], vec![hedge, fence.clone()], &[]);
        assert_eq!(
            cadastre,
            [MapObject {
                tags: cadastre[0].tags.clone(),
                ..fence
            }]
        );
    }

    #[test]
    fn two_walls_of_one_source_stay() {
        let f = fixture(MapKind::Sprint);
        // Two ALKIS walls 2 m apart: a passage between two walls, both stay.
        let a = line(&f, "515", ALKIS, "w1", &[(0.0, 0.0), (30.0, 0.0)]);
        let b = line(&f, "515", ALKIS, "w2", &[(0.0, 2.0), (30.0, 2.0)]);
        let (_, cadastre) = run(&f, vec![], vec![a.clone(), b.clone()], &[]);
        assert_eq!(cadastre, [a, b]);
    }

    #[test]
    fn lines_farther_than_the_gap_stay() {
        let f = fixture(MapKind::Sprint);
        // OSM retaining wall 3 m from an ALKIS wall: more than 0.65 mm (2.6 m), two objects.
        let osm = vec![line(
            &f,
            "513.2",
            "osm",
            "way/2",
            &[(0.0, 3.0), (30.0, 3.0)],
        )];
        let wall = line(&f, "515", ALKIS, "w1", &[(0.0, 0.0), (30.0, 0.0)]);
        let (kept, cadastre) = run(&f, osm.clone(), vec![wall.clone()], &[]);
        assert_eq!(kept, osm);
        assert_eq!(cadastre, [wall]);
    }

    #[test]
    fn partial_overlap_keeps_the_rest() {
        let f = fixture(MapKind::Sprint);
        // An OSM fence of 40 m whose first 15 m run 1 m beside an ALKIS wall.
        let osm = vec![line(&f, "518", "osm", "way/3", &[(0.0, 1.0), (40.0, 1.0)])];
        let wall = line(&f, "515", ALKIS, "w1", &[(-5.0, 0.0), (15.0, 0.0)]);
        let (osm, cadastre) = run(&f, osm, vec![wall], &[]);
        assert_eq!(osm.len(), 1, "{osm:?}");
        let rest = metres(&f, &osm[0]);
        assert!((22.0..26.5).contains(&rest), "{rest}");
        assert_eq!(tag(&cadastre[0], "freemapper:also"), Some("osm/way/3"));
    }

    #[test]
    fn fence_ending_at_a_wall_stays_whole() {
        let f = fixture(MapKind::Sprint);
        // A fence running north from a wall: perpendicular, nothing overlaps.
        let wall = line(&f, "515", ALKIS, "w1", &[(0.0, 0.0), (30.0, 0.0)]);
        let osm = vec![line(
            &f,
            "518",
            "osm",
            "way/4",
            &[(15.0, 0.0), (15.0, 20.0)],
        )];
        let (kept, _) = run(&f, osm.clone(), vec![wall], &[]);
        assert_eq!(kept, osm);
    }

    #[test]
    fn wall_along_a_building_goes() {
        let f = fixture(MapKind::Sprint);
        let building = f.projected_rect((0.0, 0.0), (20.0, 10.0));
        // An ALKIS wall 0.3 m off the building's south side, and one 3 m off it.
        let along = line(&f, "515", ALKIS, "w1", &[(0.0, -0.3), (20.0, -0.3)]);
        let apart = line(&f, "515", ALKIS, "w2", &[(0.0, -3.0), (20.0, -3.0)]);
        let (_, cadastre) = run(
            &f,
            vec![],
            vec![along, apart.clone()],
            std::slice::from_ref(&building),
        );
        assert_eq!(cadastre, [apart]);
        // A wall that runs on past the building keeps its free part.
        let longer = line(&f, "515", ALKIS, "w3", &[(0.0, -0.3), (35.0, -0.3)]);
        let (_, cadastre) = run(&f, vec![], vec![longer], &[building]);
        assert_eq!(cadastre.len(), 1);
        let rest = metres(&f, &cadastre[0]);
        assert!((14.0..15.5).contains(&rest), "{rest}");
        assert_eq!(
            tag(&cadastre[0], "freemapper:generalized"),
            Some(ALONG_BUILDING)
        );
    }

    #[test]
    fn trees_in_a_hedge_go() {
        let f = fixture(MapKind::Sprint);
        let hedge = line(&f, "410.1", "osm", "way/5", &[(0.0, 0.0), (30.0, 0.0)]);
        let osm = vec![
            hedge,
            f.tree("418", "osm", "node/1", (10.0, 0.6), None),
            f.tree("418", "osm", "node/2", (10.0, 3.0), None),
        ];
        let cadastre = vec![f.tree("417", "trees/potsdam", "t1", (20.0, -0.4), None)];
        let (osm, cadastre) = run(&f, osm, cadastre, &[]);
        assert!(cadastre.is_empty(), "{cadastre:?}");
        assert_eq!(osm.len(), 2);
        assert_eq!(
            tag(&osm[0], "freemapper:generalized"),
            Some("osm/node/1; trees/potsdam/t1")
        );
        assert_eq!(tag(&osm[1], "freemapper:source_id"), Some("node/2"));
    }

    #[test]
    fn tree_on_a_wall_gives_way() {
        let f = fixture(MapKind::Sprint);
        // A Potsdam tree 0.8 m off a terrace wall: its dot would cover the wall line.
        let wall = line(&f, "515", ALKIS, "w1", &[(0.0, 0.0), (30.0, 0.0)]);
        let on = f.tree("418", "trees/potsdam", "t1", (10.0, 0.8), None);
        let off = f.tree("418", "trees/potsdam", "t2", (20.0, 2.5), None);
        let (_, cadastre) = run(&f, vec![], vec![wall, on, off], &[]);
        assert_eq!(cadastre.len(), 2, "{cadastre:?}");
        assert_eq!(
            tag(&cadastre[0], "freemapper:generalized"),
            Some("trees/potsdam/t1")
        );
        assert_eq!(tag(&cadastre[1], "freemapper:source_id"), Some("t2"));
    }

    #[test]
    fn other_objects_keep_their_order() {
        let f = fixture(MapKind::Sprint);
        let bench = f.point("531", "osm", "node/9", (5.0, 5.0), &[]);
        let wall = line(&f, "515", "osm", "way/6", &[(0.0, 1.0), (30.0, 1.0)]);
        let other = line(&f, "505.1", "osm", "way/7", &[(0.0, 1.0), (30.0, 1.0)]);
        let osm = vec![bench.clone(), wall.clone(), other.clone()];
        let (kept, _) = run(&f, osm.clone(), vec![], &[]);
        assert_eq!(kept, osm);
    }
}
