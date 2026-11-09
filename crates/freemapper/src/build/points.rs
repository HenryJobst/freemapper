//! Point symbol generalization (ISSprOM 2019-2 §3.1, §3.3, §4.4, §4.5): the point symbols the
//! build adds keep the minimum gap on paper, a dense group of trees becomes the norm's group
//! symbol (402, 405 or 501.3), and no point is shown in an area that shall not be entered (520).

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

use geo::{
    Area as _, BooleanOps as _, BoundingRect as _, Buffer as _, Contains as _, Intersects as _,
    unary_union,
};
use geo_types::{Coord, LineString, MultiPolygon, Point, Polygon, Rect};

use crate::build::berlin::object_polygon;
use crate::build::buildings::footprint_coords;
use crate::build::dedup::{Rank, rank};
use crate::crs::BoundingBox;
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet};
use crate::project::MapKind;

/// The point symbols the build emits and their size on paper at ISSprOM 1:4000 (mm): the larger
/// of the norm's outer measure and the drawn extent (the larger side of its box) of the bundled
/// symbol. `true`: a native
/// school symbol, whose size is already the school size and is not enlarged.
pub const POINT_SIZES_MM: [(&str, f64, bool); 11] = [
    // Small fountain or well: OM 0.9.
    ("311", 0.90, false),
    // Spring: OM 1.2 × 0.6, the longer side.
    ("312", 1.20, false),
    // Prominent water feature: OM 1.2, drawn 1.45.
    ("313", 1.45, false),
    // Hydrant (school set).
    ("321", 1.30, true),
    // Prominent large tree: the white mask of the 2022 errata (1.4), the bundled set draws 1.2.
    ("417", 1.40, false),
    // Prominent bush or small tree.
    ("418", 0.80, false),
    // Cairn, memorial or small monument: OM 1.0.
    ("526", 1.00, false),
    // Prominent man-made feature – x: OM 1.2, drawn 1.264.
    ("531", 1.27, false),
    // Table, pole or small pylon, lamppost (school set).
    ("542", 1.72, true),
    ("544", 1.10, true),
    ("545", 1.40, true),
];
/// The gap between the outer edges of two point symbols on paper (mm), whatever their colours.
/// The norm names no gap between point symbols; this is its gap between line symbols (§3.3).
pub const POINT_GAP_MM: f64 = 0.15;
/// The crown diameter (ground metres) of a tree whose source gives none: the Berlin rule draws a
/// crown of 10 m or more as 417 (ADR 0051).
pub const DEFAULT_CROWN_M: [(&str, f64); 2] = [("417", 12.0), ("418", 5.0)];
/// The group region is the union of the crowns closed by this distance on paper (mm).
pub const GROUP_CLOSE_MM: f64 = 0.5;
/// A group whose crowns cover at least this share of its region is forest (405): the 402
/// screen itself leaves half the area white.
pub const FOREST_COVER: f64 = 0.5;
/// A group of fewer trees is always thinned.
pub const GROUP_MIN_TREES: usize = 3;

/// The size of the point symbol `code` on paper (mm) for the map kind: [`POINT_SIZES_MM`]
/// enlarged by [`MapKind::symbol_enlargement`] (§3.1), the native school symbols as they are.
pub fn point_size_mm(code: &str, kind: MapKind) -> Option<f64> {
    let &(_, size, native) = POINT_SIZES_MM.iter().find(|(c, _, _)| *c == code)?;
    Some(if native {
        size
    } else {
        size * kind.symbol_enlargement()
    })
}

/// The kept objects of [`generalize_points`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GeneralizedPoints {
    /// The candidates without the hidden, merged and thinned points, in their order.
    pub candidates: Vec<MapObject>,
    /// The group areas (402, 405, 501.3) as new candidates.
    pub areas: Vec<MapObject>,
    /// The map objects without the hidden and thinned points, in their order.
    pub map: Vec<MapObject>,
}

/// The `freemapper:source` of a group area.
pub const GENERALIZATION_SOURCE: &str = "generalization";
/// ISSprOM 520 area that shall not be entered: no point is shown in it.
const OUT_OF_BOUNDS_CODE: &str = "520";
/// The tree symbols, the only points with a group symbol in what the build emits.
const TREE_CODES: [&str; 2] = ["417", "418"];
/// The large tree stays a point on 402, which allows it to be added (§4.4).
const LARGE_TREE_CODE: &str = "417";
/// Corners of a crown disc.
const CROWN_SEGMENTS: usize = 32;
/// Outline simplification tolerance on paper, as for the canopies.
const SIMPLIFY_MM: f64 = 0.1;
/// A group region with at least this share on a drawn paved area is 501.3.
const PAVED_SHARE: f64 = 0.5;

/// A group symbol and its minimum area (mm²) and width (mm) on paper at 1:4000.
struct GroupSymbol {
    code: &'static str,
    min_area_mm2: f64,
    min_width_mm: f64,
}

/// 402 open land with scattered trees.
const SCATTERED_TREES: GroupSymbol = GroupSymbol {
    code: "402",
    min_area_mm2: 6.25,
    min_width_mm: 2.2,
};
/// 405 forest: the stricter minimum (openings in screens of other colours), as the build does
/// not know the surrounding screen.
const FOREST: GroupSymbol = GroupSymbol {
    code: "405",
    min_area_mm2: 1.0,
    min_width_mm: 0.3,
};
/// 501.3 paved area with scattered trees.
const PAVED_WITH_TREES: GroupSymbol = GroupSymbol {
    code: "501.3",
    min_area_mm2: 6.25,
    min_width_mm: 2.2,
};

/// A point symbol taking part in the generalization.
struct Spot {
    /// A map object (else a candidate).
    on_map: bool,
    /// Its index among the map objects or the candidates.
    index: usize,
    /// Map coordinates.
    xy: (f64, f64),
    /// Size on paper in map coordinates (1/1000 mm).
    size: f64,
    code: &'static str,
    /// The crown diameter (ground metres) of a tree, 0 for other points.
    crown: f64,
    rank: Option<Rank>,
    /// `<source>/<source_id>`.
    name: String,
    currency: Option<String>,
}

impl Spot {
    fn is_tree(&self) -> bool {
        TREE_CODES.contains(&self.code)
    }

    /// Two points closer than this (map coordinates) break the gap.
    fn min_distance(&self, other: &Spot, gap: f64) -> f64 {
        (self.size + other.size) / 2.0 + gap
    }

    fn distance(&self, other: &Spot) -> f64 {
        (self.xy.0 - other.xy.0).hypot(self.xy.1 - other.xy.1)
    }
}

/// The larger point first: map objects, then paper size, the crown, the rank, then the name and
/// the position.
fn larger_first(a: &Spot, b: &Spot) -> Ordering {
    b.on_map
        .cmp(&a.on_map)
        .then(b.size.total_cmp(&a.size))
        .then(b.crown.total_cmp(&a.crown))
        .then(b.rank.cmp(&a.rank))
        .then(a.name.cmp(&b.name))
        .then(a.xy.0.total_cmp(&b.xy.0))
        .then(a.xy.1.total_cmp(&b.xy.1))
}

fn tag<'a>(object: &'a MapObject, key: &str) -> Option<&'a str> {
    object
        .tags
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

/// A polygon in map coordinates and its bounds.
type Bounded = (Polygon<f64>, Rect<f64>);

fn bounded(polygon: Polygon<f64>) -> Option<Bounded> {
    let bounds = polygon.bounding_rect()?;
    Some((polygon, bounds))
}

/// `polygon` (map coordinates) in the map's CRS.
fn projected(polygon: &Polygon<f64>, georef: &Georeferencing) -> Polygon<f64> {
    let ring = |ring: &LineString<f64>| -> LineString<f64> {
        ring.coords()
            .map(|c| {
                let p = georef.map_to_projected(MapCoord::new(c.x as i32, c.y as i32));
                Coord {
                    x: p.easting,
                    y: p.northing,
                }
            })
            .collect()
    };
    Polygon::new(
        ring(polygon.exterior()),
        polygon.interiors().iter().map(ring).collect(),
    )
}

/// The crown of a tree: a disc of `radius` around `centre` with [`CROWN_SEGMENTS`] corners.
fn disc(centre: Coord<f64>, radius: f64) -> Polygon<f64> {
    let mut ring: Vec<Coord<f64>> = (0..CROWN_SEGMENTS)
        .map(|k| {
            let angle = std::f64::consts::TAU * k as f64 / CROWN_SEGMENTS as f64;
            Coord {
                x: centre.x + radius * angle.cos(),
                y: centre.y + radius * angle.sin(),
            }
        })
        .collect();
    ring.push(ring[0]);
    Polygon::new(LineString(ring), vec![])
}

/// The polygons among `polygons` whose bounds meet `bounds`, as one union.
fn union_near<'a>(
    polygons: impl Iterator<Item = &'a Polygon<f64>>,
    bounds: Rect<f64>,
) -> MultiPolygon<f64> {
    let near: Vec<&Polygon<f64>> = polygons
        .filter(|p| p.bounding_rect().is_some_and(|r| r.intersects(&bounds)))
        .collect();
    unary_union(near)
}

/// What the group areas are cut with and where they are drawn.
struct GroupContext<'a> {
    /// The 520 areas, the 521 footprints and the water, in the map's CRS.
    obstacles: Vec<Polygon<f64>>,
    /// The drawn paved areas in the map's CRS.
    paved: Vec<Polygon<f64>>,
    clip: Polygon<f64>,
    kind: MapKind,
    georef: &'a Georeferencing,
    symbol_set: &'a SymbolSet,
}

/// The group areas of the trees `cluster` (indices into `spots`) and the trees they take in: the
/// crowns closed by [`GROUP_CLOSE_MM`], drawn as 501.3 on paving, 405 when the crowns cover at
/// least [`FOREST_COVER`], else 402, cut at the clip window, the 520 areas, the buildings and
/// the water. Parts below the symbol's minimum are left out.
fn group_areas(
    cluster: &[usize],
    spots: &[Spot],
    context: &GroupContext<'_>,
) -> Result<(Vec<MapObject>, Vec<usize>), GeorefError> {
    let georef = context.georef;
    let enlargement = context.kind.symbol_enlargement();
    let metres_per_mm = f64::from(georef.scale_denominator) / 1000.0;
    let centre = |spot: &Spot| {
        let p = georef.map_to_projected(MapCoord::new(spot.xy.0 as i32, spot.xy.1 as i32));
        Coord {
            x: p.easting,
            y: p.northing,
        }
    };
    let crowns: Vec<Polygon<f64>> = cluster
        .iter()
        .map(|&i| disc(centre(&spots[i]), spots[i].crown / 2.0))
        .collect();
    let union = unary_union(&crowns);
    let close = GROUP_CLOSE_MM * enlargement * metres_per_mm;
    let region = union.buffer(close).buffer(-close);
    let region_area = region.unsigned_area();
    let Some(bounds) = region.bounding_rect().filter(|_| region_area > 0.0) else {
        return Ok((Vec::new(), Vec::new()));
    };
    let cover = (union.unsigned_area() / region_area).min(1.0);
    let paved = region
        .intersection(&union_near(context.paved.iter(), bounds))
        .unsigned_area();
    let symbol = if paved >= PAVED_SHARE * region_area {
        &PAVED_WITH_TREES
    } else if cover >= FOREST_COVER {
        &FOREST
    } else {
        &SCATTERED_TREES
    };
    let Some(symbol_id) = context.symbol_set.id_for_code(symbol.code) else {
        return Ok((Vec::new(), Vec::new()));
    };
    let mut cut = region.intersection(&context.clip);
    let obstacles = union_near(context.obstacles.iter(), bounds);
    if !obstacles.0.is_empty() {
        cut = cut.difference(&obstacles);
    }
    let min_area = symbol.min_area_mm2 * (enlargement * metres_per_mm).powi(2);
    // The norm's minimum width is the narrowest dimension: a disc of it must fit into the part.
    let half_width = symbol.min_width_mm * enlargement * metres_per_mm / 2.0;
    let tolerance = SIMPLIFY_MM * metres_per_mm;
    let (mut areas, mut merged) = (Vec::new(), Vec::new());
    for part in cut.0 {
        if part.unsigned_area() < min_area || part.buffer(-half_width).unsigned_area() <= 0.0 {
            continue;
        }
        // On 402 the large trees stay points (§4.4: they "may be added").
        let members: Vec<usize> = cluster
            .iter()
            .copied()
            .filter(|&i| symbol.code != SCATTERED_TREES.code || spots[i].code != LARGE_TREE_CODE)
            .filter(|&i| !merged.contains(&i) && part.intersects(&Point(centre(&spots[i]))))
            .collect();
        if members.is_empty() {
            continue;
        }
        let mut names: Vec<&str> = members.iter().map(|&i| spots[i].name.as_str()).collect();
        names.sort_unstable();
        let mut tags = vec![
            (
                "freemapper:source".to_string(),
                GENERALIZATION_SOURCE.to_string(),
            ),
            ("freemapper:source_id".to_string(), names.join("; ")),
        ];
        let oldest = members
            .iter()
            .filter_map(|&i| spots[i].currency.as_deref())
            .filter(|d| d.parse::<chrono::NaiveDate>().is_ok())
            .min();
        if let Some(oldest) = oldest {
            tags.push(("freemapper:currency".to_string(), oldest.to_string()));
        }
        tags.push((
            "freemapper:generalized".to_string(),
            format!(
                "{} trees, cover {} %",
                members.len(),
                (cover * 100.0).round()
            ),
        ));
        areas.push(MapObject {
            symbol_id,
            kind: MapObjectKind::Path,
            coords: footprint_coords(&part, min_area, tolerance, georef)?,
            tags,
        });
        merged.extend(members);
    }
    Ok((areas, merged))
}

/// The clusters of at least [`GROUP_MIN_TREES`] among the `trees` (indices into `spots`): single
/// linkage, two trees closer than their minimum distance are in one cluster. Each cluster is
/// sorted, the clusters by their first tree.
fn tree_clusters(spots: &[Spot], trees: &[usize], gap: f64) -> Vec<Vec<usize>> {
    let largest = trees.iter().map(|&i| spots[i].size).fold(0.0, f64::max);
    // A grid of cells of the largest pair distance: a close tree lies in a neighbouring cell.
    let cell = largest + gap;
    let key = |(x, y): (f64, f64)| ((x / cell).floor() as i64, (y / cell).floor() as i64);
    let mut parent: Vec<usize> = (0..trees.len()).collect();
    fn root(parent: &mut [usize], mut a: usize) -> usize {
        while parent[a] != a {
            parent[a] = parent[parent[a]];
            a = parent[a];
        }
        a
    }
    let mut grid: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (a, &i) in trees.iter().enumerate() {
        let (cx, cy) = key(spots[i].xy);
        for gx in cx - 1..=cx + 1 {
            for gy in cy - 1..=cy + 1 {
                for &b in grid.get(&(gx, gy)).into_iter().flatten() {
                    let other = &spots[trees[b]];
                    if spots[i].distance(other) < spots[i].min_distance(other, gap) {
                        let (ra, rb) = (root(&mut parent, a), root(&mut parent, b));
                        parent[ra.max(rb)] = ra.min(rb);
                    }
                }
            }
        }
        grid.entry((cx, cy)).or_default().push(a);
    }
    let mut clusters: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (a, &i) in trees.iter().enumerate() {
        let r = root(&mut parent, a);
        clusters.entry(r).or_default().push(i);
    }
    let mut clusters: Vec<Vec<usize>> = clusters
        .into_values()
        .filter(|c| c.len() >= GROUP_MIN_TREES)
        .map(|mut c| {
            c.sort_unstable();
            c
        })
        .collect();
    clusters.sort_unstable();
    clusters
}

/// Generalizes the point symbols among `candidates` and `map` (ISSprOM §3.3, §4.4, §4.5):
/// 1. a point whose centre lies in a 520 area (`out_of_bounds`, or one among `map` and
///    `candidates`) is hidden;
/// 2. at least [`GROUP_MIN_TREES`] candidate trees that keep less than the minimum gap from one
///    another become a group area (`group_areas`) where it reaches the symbol's minimum;
/// 3. every other point keeps [`POINT_GAP_MM`] (× the enlargement) between the outer edges of
///    the symbols ([`point_size_mm`]); the smaller of two points goes ([`larger_first`]), and
///    the kept one names it in `freemapper:generalized`. A candidate never drops a map point.
///
/// `paved` are the drawn paved areas, `footprints` the 521 buildings and `water_union` the
/// drawn water in the map's CRS; `clip` is the map area plus buffer. Returns the kept candidates
/// and map objects in their order and the new group areas.
#[allow(clippy::too_many_arguments)]
pub fn generalize_points(
    candidates: Vec<MapObject>,
    map: Vec<MapObject>,
    out_of_bounds: &[MapObject],
    paved: &[MapObject],
    footprints: &[Polygon<f64>],
    water_union: &MultiPolygon<f64>,
    clip: &BoundingBox,
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<GeneralizedPoints, GeorefError> {
    let is_point = |o: &MapObject| matches!(o.kind, MapObjectKind::Point { .. });
    // 1. The 520 areas hide every point in them.
    let fence_id = symbol_set.id_for_code(OUT_OF_BOUNDS_CODE);
    let fences: Vec<Bounded> = out_of_bounds
        .iter()
        .chain(&map)
        .chain(&candidates)
        .filter(|o| Some(o.symbol_id) == fence_id)
        .filter_map(object_polygon)
        .filter_map(bounded)
        .collect();
    let hidden = |o: &MapObject| {
        is_point(o)
            && o.coords.first().is_some_and(|c| {
                let p = Point::new(f64::from(c.x), f64::from(c.y));
                fences
                    .iter()
                    .any(|(fence, bounds)| bounds.intersects(&p) && fence.contains(&p))
            })
    };
    let candidates: Vec<MapObject> = candidates.into_iter().filter(|o| !hidden(o)).collect();
    let map: Vec<MapObject> = map.into_iter().filter(|o| !hidden(o)).collect();

    // The points with a size, map objects first.
    let sizes: HashMap<u32, (&'static str, f64)> = POINT_SIZES_MM
        .iter()
        .filter_map(|&(code, _, _)| {
            Some((
                symbol_set.id_for_code(code)?,
                (code, point_size_mm(code, kind)? * 1000.0),
            ))
        })
        .collect();
    let spot = |on_map: bool, index: usize, o: &MapObject| -> Option<Spot> {
        let &(code, size) = sizes.get(&o.symbol_id).filter(|_| is_point(o))?;
        let c = o.coords.first()?;
        let crown = if TREE_CODES.contains(&code) {
            tag(o, "freemapper:crown_m")
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|c| c.is_finite() && *c > 0.0)
                .or_else(|| {
                    DEFAULT_CROWN_M
                        .iter()
                        .find(|(c, _)| *c == code)
                        .map(|(_, m)| *m)
                })
                .unwrap_or(0.0)
        } else {
            0.0
        };
        Some(Spot {
            on_map,
            index,
            xy: (f64::from(c.x), f64::from(c.y)),
            size,
            code,
            crown,
            rank: rank(o),
            name: format!(
                "{}/{}",
                tag(o, "freemapper:source").unwrap_or_default(),
                tag(o, "freemapper:source_id").unwrap_or_default()
            ),
            currency: tag(o, "freemapper:currency").map(str::to_string),
        })
    };
    let spots: Vec<Spot> = map
        .iter()
        .enumerate()
        .filter_map(|(i, o)| spot(true, i, o))
        .chain(
            candidates
                .iter()
                .enumerate()
                .filter_map(|(i, o)| spot(false, i, o)),
        )
        .collect();
    let gap = POINT_GAP_MM * kind.symbol_enlargement() * 1000.0;

    // 2. Dense groups of candidate trees become group areas.
    let trees: Vec<usize> = (0..spots.len())
        .filter(|&i| !spots[i].on_map && spots[i].is_tree())
        .collect();
    let mut merged = vec![false; spots.len()];
    let mut areas = Vec::new();
    let clusters = tree_clusters(&spots, &trees, gap);
    if !clusters.is_empty() {
        let mut obstacles: Vec<Polygon<f64>> =
            fences.iter().map(|(f, _)| projected(f, georef)).collect();
        obstacles.extend(footprints.iter().cloned());
        obstacles.extend(water_union.0.iter().cloned());
        let context = GroupContext {
            obstacles,
            paved: paved
                .iter()
                .filter_map(object_polygon)
                .map(|p| projected(&p, georef))
                .collect(),
            clip: Rect::new(
                Coord {
                    x: clip.min_e,
                    y: clip.min_n,
                },
                Coord {
                    x: clip.max_e,
                    y: clip.max_n,
                },
            )
            .to_polygon(),
            kind,
            georef,
            symbol_set,
        };
        for cluster in &clusters {
            let (found, members) = group_areas(cluster, &spots, &context)?;
            areas.extend(found);
            for i in members {
                merged[i] = true;
            }
        }
    }

    // 3. Thinning: the larger point first; a point closer than the gap to a kept one goes.
    let mut order: Vec<usize> = (0..spots.len()).filter(|&i| !merged[i]).collect();
    order.sort_by(|&a, &b| larger_first(&spots[a], &spots[b]));
    let mut position = vec![usize::MAX; spots.len()];
    for (p, &i) in order.iter().enumerate() {
        position[i] = p;
    }
    let largest = spots.iter().map(|s| s.size).fold(0.0, f64::max);
    let cell = largest + gap;
    let key = |(x, y): (f64, f64)| ((x / cell).floor() as i64, (y / cell).floor() as i64);
    let mut kept: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    let mut dropped = merged;
    let mut generalized: Vec<Vec<&str>> = vec![Vec::new(); spots.len()];
    for &i in &order {
        let s = &spots[i];
        let (cx, cy) = key(s.xy);
        // The largest kept point it is too close to.
        let winner = (cx - 1..=cx + 1)
            .flat_map(|gx| (cy - 1..=cy + 1).map(move |gy| (gx, gy)))
            .flat_map(|k| kept.get(&k).into_iter().flatten())
            .copied()
            .filter(|&j| s.distance(&spots[j]) < s.min_distance(&spots[j], gap))
            .min_by_key(|&j| position[j]);
        match winner {
            Some(j) => {
                dropped[i] = true;
                generalized[j].push(&s.name);
            }
            None => kept.entry((cx, cy)).or_default().push(i),
        }
    }

    // The kept objects in their order, each naming the points it dropped.
    let mut gone = (vec![false; map.len()], vec![false; candidates.len()]);
    let mut names = (
        vec![Vec::new(); map.len()],
        vec![Vec::new(); candidates.len()],
    );
    for (i, s) in spots.iter().enumerate() {
        let (gone, names) = if s.on_map {
            (&mut gone.0, &mut names.0)
        } else {
            (&mut gone.1, &mut names.1)
        };
        gone[s.index] = dropped[i];
        names[s.index] = std::mem::take(&mut generalized[i]);
    }
    let keep = |objects: Vec<MapObject>, gone: &[bool], names: &mut [Vec<&str>]| {
        objects
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !gone[*i])
            .map(|(i, mut o)| {
                let list = &mut names[i];
                if !list.is_empty() {
                    list.sort_unstable();
                    list.dedup();
                    o.tags
                        .push(("freemapper:generalized".to_string(), list.join("; ")));
                }
                o
            })
            .collect::<Vec<_>>()
    };
    Ok(GeneralizedPoints {
        candidates: keep(candidates, &gone.1, &mut names.1),
        areas,
        map: keep(map, &gone.0, &mut names.0),
    })
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use crate::crs::LatLon;
    use crate::omap::closed_ring;

    /// A map around the Viktoriapark at the scale of `kind`, clip window ± 500 m.
    pub(crate) struct Fixture {
        pub(crate) kind: MapKind,
        pub(crate) georef: Georeferencing,
        pub(crate) set: SymbolSet,
        pub(crate) clip: BoundingBox,
    }

    pub(crate) fn fixture(kind: MapKind) -> Fixture {
        let scale = if kind == MapKind::Schoolyard {
            1000
        } else {
            4000
        };
        let georef = Georeferencing::new(
            LatLon {
                lat: 52.4876,
                lon: 13.3801,
            },
            scale,
            "2026-11-09".parse().unwrap(),
        )
        .unwrap();
        let (e, n) = (georef.ref_projected.easting, georef.ref_projected.northing);
        Fixture {
            kind,
            set: kind.symbol_set(),
            clip: BoundingBox {
                min_e: e - 500.0,
                min_n: n - 500.0,
                max_e: e + 500.0,
                max_n: n + 500.0,
            },
            georef,
        }
    }

    impl Fixture {
        /// Map coordinates per ground metre.
        pub(crate) fn metre(&self) -> f64 {
            1e6 / f64::from(self.georef.scale_denominator)
        }

        /// The map coordinate `de` m right of and `dn` m above the reference point.
        pub(crate) fn at(&self, (de, dn): (f64, f64)) -> MapCoord {
            MapCoord::new(
                (de * self.metre()).round() as i32,
                (-dn * self.metre()).round() as i32,
            )
        }

        pub(crate) fn id(&self, code: &str) -> u32 {
            self.set.id_for_code(code).unwrap()
        }

        /// A point `code` at `at` from `source` with the id `id`, dated 2025-06-01; `extra` tags
        /// replace or add tags.
        pub(crate) fn point(
            &self,
            code: &str,
            source: &str,
            id: &str,
            at: (f64, f64),
            extra: &[(&str, &str)],
        ) -> MapObject {
            let mut tags: Vec<(String, String)> = [
                ("freemapper:source", source),
                ("freemapper:source_id", id),
                ("freemapper:currency", "2025-06-01"),
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
            for (k, v) in extra {
                match tags.iter_mut().find(|(key, _)| key == k) {
                    Some(tag) => tag.1 = v.to_string(),
                    None => tags.push((k.to_string(), v.to_string())),
                }
            }
            MapObject {
                symbol_id: self.id(code),
                kind: MapObjectKind::Point { rotation_rad: 0.0 },
                coords: vec![self.at(at)],
                tags,
            }
        }

        /// A tree point with an optional crown diameter (`freemapper:crown_m`).
        pub(crate) fn tree(
            &self,
            code: &str,
            source: &str,
            id: &str,
            at: (f64, f64),
            crown: Option<f64>,
        ) -> MapObject {
            let crown = crown.map(|c| c.to_string());
            let extra: Vec<(&str, &str)> = crown
                .as_deref()
                .map(|c| ("freemapper:crown_m", c))
                .into_iter()
                .collect();
            self.point(code, source, id, at, &extra)
        }

        /// `columns` × `rows` trees `code` `spacing` m apart from (0, 0), ids `t00`, `t01`, …
        pub(crate) fn grid(
            &self,
            source: &str,
            (columns, rows): (usize, usize),
            spacing: f64,
            crown: f64,
        ) -> Vec<MapObject> {
            let mut trees = Vec::new();
            for i in 0..columns {
                for j in 0..rows {
                    let at = (i as f64 * spacing, j as f64 * spacing);
                    let id = format!("t{:02}", trees.len());
                    trees.push(self.tree("418", source, &id, at, Some(crown)));
                }
            }
            trees
        }

        /// The rectangle `w` × `h` m from (`de`, `dn`) in map coordinates.
        fn corners(&self, (de, dn): (f64, f64), (w, h): (f64, f64)) -> Vec<(i32, i32)> {
            [(de, dn), (de + w, dn), (de + w, dn + h), (de, dn + h)]
                .into_iter()
                .map(|p| {
                    let c = self.at(p);
                    (c.x, c.y)
                })
                .collect()
        }

        /// An area object `code` over the rectangle `w` × `h` m from (`de`, `dn`).
        pub(crate) fn rect(&self, code: &str, from: (f64, f64), size: (f64, f64)) -> MapObject {
            MapObject {
                symbol_id: self.id(code),
                kind: MapObjectKind::Path,
                coords: closed_ring(&self.corners(from, size)),
                tags: vec![("freemapper:source".into(), "alkis-nutzung".into())],
            }
        }

        /// The same rectangle in the map's CRS (a 521 footprint).
        pub(crate) fn projected_rect(&self, from: (f64, f64), size: (f64, f64)) -> Polygon<f64> {
            let ring: Vec<(f64, f64)> = self
                .corners(from, size)
                .into_iter()
                .map(|(x, y)| {
                    let p = self.georef.map_to_projected(MapCoord::new(x, y));
                    (p.easting, p.northing)
                })
                .collect();
            Polygon::new(ring.into(), vec![])
        }

        /// [`generalize_points`] with these inputs, no water.
        pub(crate) fn run(
            &self,
            candidates: Vec<MapObject>,
            map: Vec<MapObject>,
            out_of_bounds: &[MapObject],
            paved: &[MapObject],
            footprints: &[Polygon<f64>],
        ) -> GeneralizedPoints {
            generalize_points(
                candidates,
                map,
                out_of_bounds,
                paved,
                footprints,
                &MultiPolygon(Vec::new()),
                &self.clip,
                self.kind,
                &self.georef,
                &self.set,
            )
            .unwrap()
        }
    }

    pub(crate) fn ids(objects: &[MapObject]) -> Vec<&str> {
        objects
            .iter()
            .map(|o| tag(o, "freemapper:source_id").unwrap_or(""))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;

    const REGISTER: &str = "geoportal-berlin/anlagenbaeume";

    /// The area of an area object in ground m², and its ring in the map's CRS.
    fn projected(f: &Fixture, object: &MapObject) -> Polygon<f64> {
        let polygon = crate::build::berlin::object_polygon(object).expect("an area");
        let ring: Vec<(f64, f64)> = polygon
            .exterior()
            .coords()
            .map(|c| {
                let p = f.georef.map_to_projected(crate::omap::MapCoord::new(
                    c.x.round() as i32,
                    c.y.round() as i32,
                ));
                (p.easting, p.northing)
            })
            .collect();
        Polygon::new(ring.into(), vec![])
    }

    #[test]
    fn trees_on_out_of_bounds_are_hidden() {
        let f = fixture(MapKind::Sprint);
        let fence = f.rect("520", (0.0, 0.0), (20.0, 20.0));
        let inside = || f.tree("418", REGISTER, "in", (1.0, 10.0), None);
        let outside = || f.tree("418", REGISTER, "out", (-0.5, 10.0), None);
        let found = f.run(vec![fence.clone(), inside()], vec![], &[], &[], &[]);
        assert_eq!(found.candidates, std::slice::from_ref(&fence));
        let found = f.run(
            vec![outside()],
            vec![],
            std::slice::from_ref(&fence),
            &[],
            &[],
        );
        assert_eq!(ids(&found.candidates), ["out"]);
        let found = f.run(
            vec![inside()],
            vec![],
            std::slice::from_ref(&fence),
            &[],
            &[],
        );
        assert!(found.candidates.is_empty());
        // A 520 map object hides too.
        let found = f.run(vec![inside()], vec![fence.clone()], &[], &[], &[]);
        assert!(found.candidates.is_empty());
        assert_eq!(found.map, [fence]);
    }

    #[test]
    fn hydrant_on_out_of_bounds_is_hidden() {
        let f = fixture(MapKind::Schoolyard);
        // Everything off the school grounds is out of bounds on the schoolyard map.
        let off_grounds = f.rect("520", (0.0, 0.0), (30.0, 30.0));
        let found = f.run(
            vec![
                f.point("321", "osm", "node/1", (10.0, 10.0), &[]),
                f.point("321", "osm", "node/2", (-10.0, 10.0), &[]),
            ],
            vec![],
            &[off_grounds],
            &[],
            &[],
        );
        assert_eq!(ids(&found.candidates), ["node/2"]);
    }

    #[test]
    fn two_close_trees_keep_the_larger() {
        let f = fixture(MapKind::Sprint);
        let found = f.run(
            vec![
                f.tree("418", REGISTER, "small", (3.0, 0.0), Some(6.0)),
                f.tree("417", REGISTER, "large", (0.0, 0.0), Some(12.0)),
            ],
            vec![],
            &[],
            &[],
            &[],
        );
        assert_eq!(ids(&found.candidates), ["large"]);
        assert_eq!(
            tag(&found.candidates[0], "freemapper:generalized"),
            Some("geoportal-berlin/anlagenbaeume/small")
        );
        assert!(found.areas.is_empty());
    }

    #[test]
    fn equal_trees_keep_by_rank_then_id() {
        let f = fixture(MapKind::Sprint);
        let found = f.run(
            vec![
                f.tree("418", "osm", "node/1", (0.0, 0.0), None),
                f.tree(
                    "418",
                    "geoportal-berlin/strassenbaeume",
                    "z",
                    (3.0, 0.0),
                    None,
                ),
            ],
            vec![],
            &[],
            &[],
            &[],
        );
        assert_eq!(ids(&found.candidates), ["z"]);
        assert_eq!(
            tag(&found.candidates[0], "freemapper:generalized"),
            Some("osm/node/1")
        );
        let found = f.run(
            vec![
                f.tree("418", "osm", "node/2", (0.0, 0.0), None),
                f.tree("418", "osm", "node/1", (3.0, 0.0), None),
            ],
            vec![],
            &[],
            &[],
            &[],
        );
        assert_eq!(ids(&found.candidates), ["node/1"]);
    }

    #[test]
    fn chain_of_trees_keeps_the_ends() {
        let f = fixture(MapKind::Sprint);
        let found = f.run(
            vec![
                f.tree("418", REGISTER, "a", (0.0, 0.0), Some(1.0)),
                f.tree("418", REGISTER, "b", (3.5, 0.0), Some(0.9)),
                f.tree("418", REGISTER, "c", (7.0, 0.0), Some(1.0)),
            ],
            vec![],
            &[],
            &[],
            &[],
        );
        assert!(found.areas.is_empty());
        assert_eq!(ids(&found.candidates), ["a", "c"]);
        assert_eq!(
            tag(&found.candidates[0], "freemapper:generalized"),
            Some("geoportal-berlin/anlagenbaeume/b")
        );
        assert_eq!(tag(&found.candidates[1], "freemapper:generalized"), None);
        // With b the largest, b stays and a and c go.
        let found = f.run(
            vec![
                f.tree("418", REGISTER, "a", (0.0, 0.0), Some(0.9)),
                f.tree("418", REGISTER, "b", (3.5, 0.0), Some(1.0)),
                f.tree("418", REGISTER, "c", (7.0, 0.0), Some(0.9)),
            ],
            vec![],
            &[],
            &[],
            &[],
        );
        assert_eq!(ids(&found.candidates), ["b"]);
        assert_eq!(
            tag(&found.candidates[0], "freemapper:generalized"),
            Some("geoportal-berlin/anlagenbaeume/a; geoportal-berlin/anlagenbaeume/c")
        );
    }

    /// The cover in per cent of a group area's `freemapper:generalized` (`9 trees, cover 87 %`).
    fn cover(area: &MapObject) -> u32 {
        let text = tag(area, "freemapper:generalized").expect("the group");
        let (_, cover) = text.split_once("cover ").expect("a cover");
        cover.trim_end_matches(" %").parse().expect("a number")
    }

    #[test]
    fn dense_group_becomes_forest_area() {
        let f = fixture(MapKind::Sprint);
        let mut trees = f.grid(REGISTER, (3, 3), 3.5, 6.0);
        trees.reverse();
        trees[4]
            .tags
            .iter_mut()
            .find(|(k, _)| k == "freemapper:currency")
            .unwrap()
            .1 = "2019-05-01".to_string();
        let found = f.run(trees, vec![], &[], &[], &[]);
        assert!(found.candidates.is_empty());
        assert_eq!(found.areas.len(), 1);
        let area = &found.areas[0];
        assert_eq!(area.symbol_id, f.id("405"));
        assert_eq!(area.kind, MapObjectKind::Path);
        assert!(cover(area) >= 50, "{:?}", area.tags);
        assert_eq!(tag(area, "freemapper:source"), Some(GENERALIZATION_SOURCE));
        let merged: Vec<String> = (0..9)
            .map(|i| format!("geoportal-berlin/anlagenbaeume/t{i:02}"))
            .collect();
        assert_eq!(
            tag(area, "freemapper:source_id"),
            Some(merged.join("; ").as_str())
        );
        assert_eq!(tag(area, "freemapper:currency"), Some("2019-05-01"));
        assert!(
            tag(area, "freemapper:generalized")
                .unwrap()
                .starts_with("9 trees, cover ")
        );
    }

    #[test]
    fn sparse_group_becomes_scattered_trees_area() {
        let f = fixture(MapKind::Sprint);
        let found = f.run(f.grid(REGISTER, (5, 4), 3.5, 2.0), vec![], &[], &[], &[]);
        assert!(found.candidates.is_empty());
        assert_eq!(found.areas.len(), 1);
        let area = &found.areas[0];
        assert_eq!(area.symbol_id, f.id("402"));
        assert!(projected(&f, area).unsigned_area() >= 100.0);
        assert!(cover(area) < 50, "{:?}", area.tags);
    }

    #[test]
    fn group_on_paved_area_becomes_paved_with_trees() {
        let f = fixture(MapKind::Sprint);
        let square = f.rect("501.5", (-10.0, -10.0), (40.0, 40.0));
        let found = f.run(
            f.grid(REGISTER, (5, 4), 3.5, 2.0),
            vec![],
            &[],
            &[square],
            &[],
        );
        assert!(found.candidates.is_empty());
        assert_eq!(found.areas.len(), 1);
        assert_eq!(found.areas[0].symbol_id, f.id("501.3"));
    }

    #[test]
    fn large_tree_stays_on_scattered_trees_area() {
        let f = fixture(MapKind::Sprint);
        let mut trees = f.grid(REGISTER, (5, 4), 3.5, 2.0);
        trees.push(f.tree("417", REGISTER, "large", (5.25, 5.25), Some(3.0)));
        let found = f.run(trees, vec![], &[], &[], &[]);
        assert_eq!(found.areas.len(), 1);
        assert_eq!(found.areas[0].symbol_id, f.id("402"));
        assert_eq!(ids(&found.candidates), ["large"]);
        assert!(
            !tag(&found.areas[0], "freemapper:source_id")
                .unwrap()
                .contains("large")
        );
        assert!(
            tag(&found.areas[0], "freemapper:generalized")
                .unwrap()
                .starts_with("20 trees, ")
        );
    }

    #[test]
    fn group_area_below_minimum_is_dropped() {
        let f = fixture(MapKind::Sprint);
        let h = 3.0 * 3f64.sqrt() / 2.0;
        let found = f.run(
            vec![
                f.tree("418", REGISTER, "a", (0.0, 0.0), Some(1.0)),
                f.tree("418", REGISTER, "b", (3.0, 0.0), Some(1.0)),
                f.tree("418", REGISTER, "c", (1.5, h), Some(1.0)),
            ],
            vec![],
            &[],
            &[],
            &[],
        );
        assert!(found.areas.is_empty());
        assert_eq!(ids(&found.candidates), ["a"]);
    }

    #[test]
    fn group_area_is_cut_at_buildings_and_out_of_bounds() {
        let f = fixture(MapKind::Sprint);
        // A 5 × 5 forest of 15.5 m; a building over its right part, an out-of-bounds area over
        // its top left corner.
        let trees = f.grid(REGISTER, (5, 5), 3.5, 6.0);
        let building = f.projected_rect((9.0, -5.0), (20.0, 30.0));
        let fence = f.rect("520", (-10.0, 9.0), (12.0, 20.0));
        let found = f.run(
            trees,
            vec![],
            std::slice::from_ref(&fence),
            &[],
            std::slice::from_ref(&building),
        );
        assert!(!found.areas.is_empty());
        let fence = projected(&f, &fence);
        for area in &found.areas {
            assert_eq!(area.symbol_id, f.id("405"));
            let area = projected(&f, area);
            assert!(area.intersection(&building).unsigned_area() < 0.01);
            assert!(area.intersection(&fence).unsigned_area() < 0.01);
        }
        // The trees under the building stay points, thinned; none is left in the 520.
        let left: Vec<(f64, f64)> = found
            .candidates
            .iter()
            .map(|o| {
                let p = f.georef.map_to_projected(o.coords[0]);
                (p.easting, p.northing)
            })
            .collect();
        use geo::Contains as _;
        assert!(!left.is_empty());
        assert!(
            left.iter()
                .all(|&(e, n)| building.contains(&geo_types::Point::new(e, n)))
        );
        assert!(
            left.iter()
                .all(|&(e, n)| !fence.contains(&geo_types::Point::new(e, n)))
        );
        // Under the building the columns at 10.5 and 14 m are 3.5 m apart: thinned.
        assert!(left.len() < 10, "{left:?}");
    }

    #[test]
    fn trees_far_apart_are_unchanged() {
        let f = fixture(MapKind::Sprint);
        let mut trees: Vec<MapObject> = (0..5)
            .map(|i| {
                let id = format!("s{i}");
                f.tree("418", REGISTER, &id, (f64::from(i) * 10.0, 0.0), None)
            })
            .collect();
        trees.extend((0..5).map(|i| {
            let id = format!("l{i}");
            f.tree("417", REGISTER, &id, (f64::from(i) * 7.0, 20.0), None)
        }));
        let found = f.run(trees.clone(), vec![], &[], &[], &[]);
        assert_eq!(found.candidates, trees);
        assert!(found.areas.is_empty());
    }

    #[test]
    fn schoolyard_scale_uses_enlarged_symbols() {
        let pair = |f: &Fixture, apart: f64| {
            let found = f.run(
                vec![
                    f.tree("418", REGISTER, "a", (0.0, 0.0), None),
                    f.tree("418", REGISTER, "b", (apart, 0.0), None),
                ],
                vec![],
                &[],
                &[],
                &[],
            );
            found.candidates.len()
        };
        assert_eq!(pair(&fixture(MapKind::Sprint), 2.0), 1);
        let school = fixture(MapKind::Schoolyard);
        assert_eq!(pair(&school, 2.0), 2);
        assert_eq!(pair(&school, 1.1), 1);
    }

    #[test]
    fn close_bollards_keep_one() {
        let f = fixture(MapKind::Schoolyard);
        let pair = |apart: f64| {
            f.run(
                vec![
                    f.point("544", "osm", "node/1", (0.0, 0.0), &[]),
                    f.point("544", "geoportal-berlin/sb-moebel", "m", (apart, 0.0), &[]),
                    f.point("544", "osm", "node/2", (0.0, 20.0), &[]),
                    f.point("544", "osm", "node/3", (apart, 20.0), &[]),
                ],
                vec![],
                &[],
                &[],
                &[],
            )
        };
        assert_eq!(ids(&pair(1.0).candidates), ["m", "node/2"]);
        assert_eq!(pair(1.4).candidates.len(), 4);
    }

    #[test]
    fn different_point_symbols_too_close_keep_the_larger() {
        let f = fixture(MapKind::Schoolyard);
        let found = f.run(
            vec![
                f.point("544", "osm", "node/1", (0.0, 0.0), &[]),
                f.point("545", "osm", "node/2", (1.0, 0.0), &[]),
            ],
            vec![],
            &[],
            &[],
            &[],
        );
        assert_eq!(ids(&found.candidates), ["node/2"]);
        let f = fixture(MapKind::Sprint);
        let found = f.run(
            vec![
                f.tree("418", REGISTER, "tree", (0.0, 0.0), None),
                f.point("531", "osm", "node/1", (3.0, 0.0), &[]),
            ],
            vec![],
            &[],
            &[],
            &[],
        );
        assert_eq!(ids(&found.candidates), ["node/1"]);
    }

    #[test]
    fn map_point_beats_candidates() {
        let f = fixture(MapKind::Sprint);
        let found = f.run(
            vec![f.tree("417", REGISTER, "large", (3.0, 0.0), Some(14.0))],
            vec![
                f.tree("418", "osm", "node/1", (0.0, 0.0), None),
                f.tree("418", "osm", "node/2", (0.0, 2.0), None),
            ],
            &[],
            &[],
            &[],
        );
        assert!(found.candidates.is_empty());
        assert_eq!(ids(&found.map), ["node/1"]);
        assert_eq!(
            tag(&found.map[0], "freemapper:generalized"),
            Some("geoportal-berlin/anlagenbaeume/large; osm/node/2")
        );
    }

    /// The larger side (mm) of the box that holds all a point symbol draws.
    fn drawn_extent_mm(set: &SymbolSet, code: &str) -> f64 {
        use crate::render::symbols::{PointDef, RenderSymbols, Symbol};
        /// (min x, min y, max x, max y) of what `point` draws around (`dx`, `dy`).
        fn bounds(point: &PointDef, (dx, dy): (f64, f64), b: &mut [f64; 4]) {
            fn add(b: &mut [f64; 4], x: f64, y: f64, half: f64) {
                b[0] = b[0].min(x - half);
                b[1] = b[1].min(y - half);
                b[2] = b[2].max(x + half);
                b[3] = b[3].max(y + half);
            }
            if let Some((_, inner)) = point.dot {
                add(b, dx, dy, inner);
            }
            if let Some((_, centre, width)) = point.ring {
                add(b, dx, dy, centre + width / 2.0);
            }
            for element in &point.elements {
                for c in &element.coords {
                    let (x, y) = (dx + f64::from(c.x), dy + f64::from(c.y));
                    match &element.symbol {
                        Symbol::Line(line) => add(b, x, y, line.width / 2.0),
                        Symbol::Point(inner) => bounds(inner, (x, y), b),
                        _ => add(b, x, y, 0.0),
                    }
                }
            }
        }
        let symbols = RenderSymbols::from_set(set);
        let Symbol::Point(point) = &symbols.symbols[&set.id_for_code(code).unwrap()] else {
            panic!("{code} is no point symbol");
        };
        let mut b = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
        bounds(point, (0.0, 0.0), &mut b);
        (b[2] - b[0]).max(b[3] - b[1]) / 1000.0
    }

    #[test]
    fn point_sizes_cover_the_drawn_symbols() {
        for kind in [MapKind::Sprint, MapKind::Schoolyard] {
            let set = kind.symbol_set();
            for (code, _, _) in POINT_SIZES_MM {
                if set.id_for_code(code).is_none() {
                    continue;
                }
                let size = point_size_mm(code, kind).expect("in the table");
                let drawn = drawn_extent_mm(&set, code);
                // The enlarged school set rounds to whole micrometres.
                assert!(size >= drawn - 0.002, "{kind:?} {code}: {size} < {drawn}");
            }
        }
        // 417 at the norm's white mask, not the bundled 1.2 mm.
        assert!((point_size_mm("417", MapKind::Sprint).unwrap() - 1.4).abs() < 1e-9);
        assert!((point_size_mm("417", MapKind::Schoolyard).unwrap() - 1.75).abs() < 1e-9);
        assert!((point_size_mm("544", MapKind::Schoolyard).unwrap() - 1.1).abs() < 1e-9);
        use crate::build::candidates::Rules;
        for rules in [Rules::embedded(), Rules::berlin(), Rules::states()] {
            for code in rules.point_codes() {
                assert!(
                    POINT_SIZES_MM.iter().any(|(c, _, _)| *c == code),
                    "{code} has no size"
                );
            }
        }
    }
}
