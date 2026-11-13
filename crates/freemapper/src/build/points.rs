//! Point symbol generalization (ISSprOM 2019-2 §3.1, §3.3, §4.4, §4.5): the point symbols the
//! build adds keep the minimum gap on paper, a dense group of trees becomes the norm's group
//! symbol (402, 405 or 501.3, cut at paving and paths; on a white base, a map without the walkable
//! area, a 405 gets a 416 boundary so that it shows), and no point is shown in an area that shall
//! not be entered (520).

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

use geo::{
    Area as _, BooleanOps as _, BoundingRect as _, Buffer as _, Contains as _, Euclidean,
    Intersects as _, Length as _, Simplify as _, unary_union,
};
use geo_types::{Coord, LineString, MultiLineString, MultiPolygon, Point, Polygon, Rect};

use crate::build::buildings::footprint_coords;
use crate::build::control_sites::{CONTROL_KEY, drop_ambiguous as drop_ambiguous_control_sites};
use crate::build::dedup::{Rank, rank};
use crate::build::geometry::object_polygon;
use crate::crs::{BoundingBox, ProjectedPoint};
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet, closed_ring};
use crate::project::MapKind;

/// The point symbols the build emits and their size on paper at ISSprOM 1:4000 (mm): the larger
/// of the norm's outer measure and the drawn extent (the larger side of its box) of the bundled
/// symbol. `true`: a native
/// school symbol, whose size is already the school size and is not enlarged.
pub const POINT_SIZES_MM: [(&str, f64, bool); 13] = [
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
    // High tower: OM 2.1, drawn 2.35.
    ("524", 2.35, false),
    // Small tower: OM 1.5, drawn 1.72.
    ("525", 1.72, false),
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
/// Two trees whose crowns come closer than this on paper (mm) belong to one group: their crowns
/// touch or nearly touch, a closed canopy, which ISSprOM maps as forest (§4.4, 405). The group
/// region is closed by the same distance.
pub const GROUP_CLOSE_MM: f64 = 0.5;
/// Two trees closer than this on paper (mm, centre to centre) also belong to one group, and each
/// tree claims a disc of half this diameter (or its crown, if larger) for the group region. It is
/// the side of the smallest 402 (6.25 mm², §4.4): trees closer than that read as a dot field, not
/// as single features, and their group is at least as large as the smallest 402. At 1:4000 it is
/// 10 m. The norm names no spacing; this is a ruling (2026-11-10).
pub const GROUP_LINK_MM: f64 = 2.5;
/// The opening of a group region is at least this many times the mean claim diameter of its
/// trees: a single row (one claim wide) joined to a group falls out of the area and stays points.
const ROW_OPENING: f64 = 1.2;
/// A group whose crowns cover at least this share of its region is forest (405): the 402
/// screen itself leaves half the area white.
pub const FOREST_COVER: f64 = 0.5;
/// A group of fewer trees is always thinned, and so is the part of a group area that takes in
/// fewer.
pub const GROUP_MIN_TREES: usize = 3;
/// A 501.3 takes in at least this many trees (a square planted three by three): the few large
/// street trees whose crowns meet over a pavement stay points.
const PAVED_MIN_TREES: usize = 9;

/// The size of the point symbol `code` on paper (mm) for the map kind: [`POINT_SIZES_MM`]
/// enlarged by [`MapKind::symbol_enlargement`] (§3.1), the native school symbols as they are.
pub fn point_size_mm(code: &str, kind: MapKind, scale: u32) -> Option<f64> {
    let &(_, size, native) = POINT_SIZES_MM.iter().find(|(c, _, _)| *c == code)?;
    Some(if native {
        size
    } else {
        size * kind.symbol_enlargement(scale)
    })
}

/// The kept objects of [`generalize_points`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GeneralizedPoints {
    /// The candidates without the hidden, merged and thinned points, in their order.
    pub candidates: Vec<MapObject>,
    /// The group areas (402, 405, 501.3) and the 416 boundaries of a 405 on the white base as
    /// new candidates.
    pub areas: Vec<MapObject>,
    /// The map objects without the hidden and thinned points, in their order.
    pub map: Vec<MapObject>,
}

/// The `freemapper:source` of a group area.
pub const GENERALIZATION_SOURCE: &str = "generalization";
/// The tag of a candidate point under a 520 candidate: it stays in the candidates, marked with
/// the code, and is left out of the training map.
pub const HIDDEN_KEY: &str = "freemapper:hidden";
/// ISSprOM 520 area that shall not be entered: no point is shown in it.
pub(crate) const OUT_OF_BOUNDS_CODE: &str = "520";
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
/// Off paving, a group region with at least this share on open land ([`OPEN_LAND_CODES`]) is a
/// plain 405 (or 402): the yellow round it shows the white forest. Elsewhere the base is white,
/// and a 405 gets a 416 boundary ([`BOUNDED_FOREST`]).
const OPEN_LAND_SHARE: f64 = 0.5;
/// ISSprOM open land: 401 open land, 402 with scattered trees, 403 rough open land, 404 with
/// scattered trees.
const OPEN_LAND_CODES: [&str; 4] = ["401", "402", "403", "404"];
/// A cluster whose trees lie in a corridor narrower than this many symbol sizes (the mean of
/// its trees) is a row ...
const ROW_WIDTH_SIZES: f64 = 2.0;
/// ... and so is a cluster at least this many times as long as it is wide (along its main axis,
/// one symbol size added to both; on paving any such cluster, a street with its trees) ...
const ROW_ASPECT: f64 = 3.0;
/// ... as long as it is narrower than this many mean crown diameters: an avenue, also one with
/// trees on both sides of a narrow street, stays a row of tree symbols, never a strip of area,
/// while a long park strip several crowns deep is a group.
const ROW_MAX_WIDTH_CROWNS: f64 = 2.0;
/// 416 distinct vegetation boundary: "a distinct forest edge" (§4.4). It outlines a 405 on the
/// white base, where the forest's white alone shows nothing.
const VEGETATION_BOUNDARY_CODE: &str = "416";
/// The shortest 416 piece on paper (mm): the norm's minimum length of an isolated 416, five
/// dots (2.65 mm, §4.4).
const BOUNDARY_MIN_MM: f64 = 2.65;
/// No 416 is drawn within this distance on paper (mm) of an edge where the area was cut (a
/// building, water, a 520 area, a paved area, the clip window): there the cut feature is the
/// boundary.
const BOUNDARY_CUT_MM: f64 = 0.1;
/// The paths a 402 or 405 is cut at, with their width on paper (mm) at 1:4000: the line of the
/// bundled symbol and, for 505, the 501 footprints and the 509.1 railway, its two 0.1 mm border
/// lines. A forest never
/// covers a path (ADR 0069).
const PATH_WIDTHS_MM: [(&str, f64); 9] = [
    ("505.1", 0.55),
    ("505.2", 0.70),
    ("506", 0.27),
    ("507", 0.27),
    ("501.6", 0.55),
    ("501.7", 0.70),
    ("501.8", 1.0),
    ("501.9", 1.2),
    ("509.1", 0.55),
];
/// The paved areas among the candidates a 402 or 405 is cut at, like the drawn paving.
const PAVED_AREA_CODES: [&str; 5] = ["501", "501.5", "501.10", "501.11", "501.3"];
/// The cell of the grid indexes of the areas (ground metres).
const GRID_CELL_M: f64 = 100.0;
/// An area spanning more cells than this is checked for every query instead.
const GRID_MAX_CELLS: i64 = 4096;

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
/// 405 forest on the white base, outlined by a 416 dotted line: the smallest area a dotted line
/// may enclose, 1.5 mm across (§3.3), so 1.77 mm² and 1.5 mm wide.
const BOUNDED_FOREST: GroupSymbol = GroupSymbol {
    code: "405",
    min_area_mm2: 1.77,
    min_width_mm: 1.5,
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
    /// The crown diameter (ground metres) of a tree ([`DEFAULT_CROWN_M`] when the source gives
    /// none), 0 for other points.
    crown: f64,
    /// The source gives the crown.
    crown_measured: bool,
    /// A control site (ADR 0074): it gives way to every other point.
    weak: bool,
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

/// The larger point first: map objects, then the points that are no control sites, then paper size, the rank, a measured crown before a
/// default one, the crown, then the name and the position. The rank comes before the crown so
/// that a default crown never beats a measured one of a better source, and the order stays total.
fn larger_first(a: &Spot, b: &Spot) -> Ordering {
    b.on_map
        .cmp(&a.on_map)
        .then(a.weak.cmp(&b.weak))
        .then(b.size.total_cmp(&a.size))
        .then(b.rank.cmp(&a.rank))
        .then(b.crown_measured.cmp(&a.crown_measured))
        .then(b.crown.total_cmp(&a.crown))
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

/// A grid index of bounding boxes: the items whose box meets each cell of `cell` size.
struct BoxGrid {
    cell: f64,
    cells: HashMap<(i64, i64), Vec<usize>>,
    /// Items too large for the grid ([`GRID_MAX_CELLS`]), checked for every query.
    wide: Vec<usize>,
}

impl BoxGrid {
    fn new<'a>(polygons: impl Iterator<Item = &'a Polygon<f64>>, cell: f64) -> Self {
        let mut grid = Self {
            cell,
            cells: HashMap::new(),
            wide: Vec::new(),
        };
        for (i, polygon) in polygons.enumerate() {
            let Some(bounds) = polygon.bounding_rect() else {
                continue;
            };
            let ((x0, y0), (x1, y1)) = grid.span(bounds);
            if (x1 - x0 + 1) * (y1 - y0 + 1) > GRID_MAX_CELLS {
                grid.wide.push(i);
                continue;
            }
            for x in x0..=x1 {
                for y in y0..=y1 {
                    grid.cells.entry((x, y)).or_default().push(i);
                }
            }
        }
        grid
    }

    fn span(&self, bounds: Rect<f64>) -> ((i64, i64), (i64, i64)) {
        let key = |c: Coord<f64>| {
            (
                (c.x / self.cell).floor() as i64,
                (c.y / self.cell).floor() as i64,
            )
        };
        (key(bounds.min()), key(bounds.max()))
    }

    /// The items whose cells meet `bounds`, sorted, each once.
    fn near(&self, bounds: Rect<f64>) -> Vec<usize> {
        let ((x0, y0), (x1, y1)) = self.span(bounds);
        let mut found = self.wide.clone();
        for x in x0..=x1 {
            for y in y0..=y1 {
                found.extend(self.cells.get(&(x, y)).into_iter().flatten());
            }
        }
        found.sort_unstable();
        found.dedup();
        found
    }
}

/// Polygons with a grid index.
struct Indexed {
    polygons: Vec<Polygon<f64>>,
    grid: BoxGrid,
}

impl Indexed {
    fn new(polygons: Vec<Polygon<f64>>, cell: f64) -> Self {
        let grid = BoxGrid::new(polygons.iter(), cell);
        Self { polygons, grid }
    }

    /// Whether `p` lies inside one of the polygons.
    fn contains(&self, p: Point<f64>) -> bool {
        self.grid
            .near(Rect::new(p.0, p.0))
            .into_iter()
            .any(|i| self.polygons[i].contains(&p))
    }

    /// The union of the polygons whose bounds meet `bounds`.
    fn union_near(&self, bounds: Rect<f64>) -> MultiPolygon<f64> {
        let near: Vec<&Polygon<f64>> = self
            .grid
            .near(bounds)
            .into_iter()
            .map(|i| &self.polygons[i])
            .filter(|p| p.bounding_rect().is_some_and(|r| r.intersects(&bounds)))
            .collect();
        unary_union(near)
    }
}

/// `polygon` (map coordinates) in the map's CRS.
pub(crate) fn projected(polygon: &Polygon<f64>, georef: &Georeferencing) -> Polygon<f64> {
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

/// What the group areas are cut with and where they are drawn.
struct GroupContext<'a> {
    /// The 520 areas, the 521 footprints and the water, in the map's CRS.
    obstacles: Indexed,
    /// The drawn paved areas in the map's CRS.
    paved: Indexed,
    /// The paved candidates and the path footprints in the map's CRS: a 402 or 405 is cut at
    /// them, like at the drawn paving.
    ways: Indexed,
    /// The map has no walkable area (schoolyard): its base is white, and a 405 off open land
    /// needs a 416 boundary.
    white_base: bool,
    /// The open land (401 to 404) among the map objects and candidates in the map's CRS.
    open_land: Indexed,
    clip: Polygon<f64>,
    kind: MapKind,
    georef: &'a Georeferencing,
    symbol_set: &'a SymbolSet,
}

/// The group areas of the trees `cluster` (indices into `spots`) and the trees they take in: the
/// crowns closed by [`GROUP_CLOSE_MM`], drawn as 501.3 on paving; elsewhere 405 when the crowns
/// cover at least [`FOREST_COVER`] and 402 when not. A 405 off open land ([`OPEN_LAND_SHARE`]) on
/// a map without the walkable area lies on the white base and gets a 416 boundary
/// ([`BOUNDED_FOREST`]). The shares are measured
/// on the closed crowns before the cuts. The area is cut at the clip window, the 520 areas, the
/// buildings and the water, and a 402 or 405 also at the drawn paved areas, the paved candidates
/// and the paths ([`way_footprints`]). Each part is opened by
/// half the symbol's minimum width, which leaves out what is narrower (the norm's minimum width is
/// that of the narrowest part); pieces below the minimum area are left out. A tree is merged when
/// its centre lies in the part and its crown meets a kept piece.
fn group_areas(
    cluster: &[usize],
    spots: &[Spot],
    context: &GroupContext<'_>,
) -> Result<(Vec<MapObject>, Vec<usize>), GeorefError> {
    let georef = context.georef;
    let enlargement = context.kind.symbol_enlargement(georef.scale_denominator);
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
    // Each tree claims at least half the link distance round it, so that scattered trees make
    // one region too.
    let close = GROUP_CLOSE_MM * enlargement * metres_per_mm;
    let region_of = |reach: f64| {
        let radii: Vec<f64> = cluster
            .iter()
            .map(|&i| (spots[i].crown / 2.0).max(reach))
            .collect();
        let claims: Vec<Polygon<f64>> = cluster
            .iter()
            .zip(&radii)
            .map(|(&i, &r)| disc(centre(&spots[i]), r))
            .collect();
        let region = unary_union(&claims).buffer(close).buffer(-close);
        (region, radii)
    };
    let (mut region, mut radii) = region_of(GROUP_LINK_MM * enlargement * metres_per_mm / 2.0);
    let Some(bounds) = region
        .bounding_rect()
        .filter(|_| region.unsigned_area() > 0.0)
    else {
        return Ok((Vec::new(), Vec::new()));
    };
    let paved_near = context.paved.union_near(bounds);
    let paved = region.intersection(&paved_near).unsigned_area();
    let on_paving = paved >= PAVED_SHARE * region.unsigned_area();
    if on_paving {
        // A long group on paving is a street with its rows of trees, no square: it stays points.
        let (length, width, size, _) = extent(cluster, spots);
        if is_elongated(length, width, size) {
            return Ok((Vec::new(), Vec::new()));
        }
        // On paving the trees claim only their crowns: the trees of a square stand close, the
        // rows along a street are one crown wide and fall to the opening.
        (region, radii) = region_of(0.0);
    }
    let region_area = region.unsigned_area();
    if region_area <= 0.0 {
        return Ok((Vec::new(), Vec::new()));
    }
    // The cover: the share of the region under the crowns.
    let cover = (union.intersection(&region).unsigned_area() / region_area).min(1.0);
    let (symbol, bounded) = if on_paving {
        (&PAVED_WITH_TREES, false)
    } else if cover < FOREST_COVER {
        (&SCATTERED_TREES, false)
    } else {
        let open = region
            .intersection(&context.open_land.union_near(bounds))
            .unsigned_area();
        if !context.white_base || open >= OPEN_LAND_SHARE * region_area {
            (&FOREST, false)
        } else {
            (&BOUNDED_FOREST, true)
        }
    };
    let Some(symbol_id) = context.symbol_set.id_for_code(symbol.code) else {
        return Ok((Vec::new(), Vec::new()));
    };
    let boundary_id = context
        .symbol_set
        .id_for_code(VEGETATION_BOUNDARY_CODE)
        .filter(|_| bounded);
    let mut cut = region.intersection(&context.clip);
    let mut obstacles = context.obstacles.union_near(bounds);
    // A road, square or path through the trees is no forest: off paving the drawn paving, the
    // paved candidates and the paths are cut out, so that no group area covers them.
    if !on_paving {
        for cut_out in [paved_near, context.ways.union_near(bounds)] {
            if cut_out.0.is_empty() {
                continue;
            }
            obstacles = if obstacles.0.is_empty() {
                cut_out
            } else {
                obstacles.union(&cut_out)
            };
        }
    }
    if !obstacles.0.is_empty() {
        cut = cut.difference(&obstacles);
    }
    let min_area = symbol.min_area_mm2 * (enlargement * metres_per_mm).powi(2);
    // The opening is at least as wide as a single row of the cluster's trees (its mean claim
    // times [`ROW_OPENING`]), so that a row joined to a group stays out of the area.
    let row = ROW_OPENING * 2.0 * radii.iter().sum::<f64>() / radii.len() as f64;
    let half_width = (symbol.min_width_mm * enlargement * metres_per_mm).max(row) / 2.0;
    let tolerance = SIMPLIFY_MM * metres_per_mm;
    // The opening: what is left after eroding and dilating by half the minimum width.
    let pieces = cut.0.into_iter().flat_map(|part| {
        let opened = part.buffer(-half_width).buffer(half_width);
        let opened = opened.intersection(&part);
        opened.0.into_iter().map(move |piece| (part.clone(), piece))
    });
    // Where the boundary is a cut: within reach of an obstacle or of the clip window's edge.
    let reach = BOUNDARY_CUT_MM * enlargement * metres_per_mm;
    let no_boundary = boundary_id.map(|_| {
        let rim = MultiPolygon(vec![context.clip.clone()])
            .difference(&MultiPolygon(vec![context.clip.clone()]).buffer(-reach));
        if obstacles.0.is_empty() {
            rim
        } else {
            rim.union(&obstacles.buffer(reach))
        }
    });
    let (mut areas, mut merged) = (Vec::new(), Vec::new());
    for (part, piece) in pieces {
        if piece.unsigned_area() < min_area {
            continue;
        }
        // On 402 the large trees stay points (§4.4: they "may be added").
        let members: Vec<usize> = cluster
            .iter()
            .copied()
            .zip(&crowns)
            .filter(|&(i, _)| {
                symbol.code != SCATTERED_TREES.code || spots[i].code != LARGE_TREE_CODE
            })
            .filter(|&(i, crown)| {
                !merged.contains(&i)
                    && part.intersects(&Point(centre(&spots[i])))
                    && piece.intersects(crown)
            })
            .map(|(i, _)| i)
            .collect();
        let least = if on_paving {
            PAVED_MIN_TREES
        } else {
            GROUP_MIN_TREES
        };
        if members.len() < least {
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
        if let (Some(boundary_id), Some(no_boundary)) = (boundary_id, &no_boundary) {
            let length = BOUNDARY_MIN_MM * enlargement * metres_per_mm;
            for line in boundary_lines(&piece, no_boundary, length, tolerance) {
                areas.push(MapObject {
                    symbol_id: boundary_id,
                    kind: MapObjectKind::Path,
                    coords: line_coords(&line, georef)?,
                    tags: tags.clone(),
                });
            }
        }
        areas.push(MapObject {
            symbol_id,
            kind: MapObjectKind::Path,
            coords: footprint_coords(&piece, min_area, tolerance, georef)?,
            tags,
        });
        merged.extend(members);
    }
    Ok((areas, merged))
}

/// The paved area candidates ([`PAVED_AREA_CODES`] among `areas` and `candidates`) and the
/// footprints of the paths ([`PATH_WIDTHS_MM`] among `candidates` and `map`, buffered by half
/// their width on paper) in the map's CRS.
pub(crate) fn way_footprints(
    areas: &[MapObject],
    candidates: &[MapObject],
    map: &[MapObject],
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Vec<Polygon<f64>> {
    let paved_ids: Vec<u32> = PAVED_AREA_CODES
        .iter()
        .filter_map(|code| symbol_set.id_for_code(code))
        .collect();
    let metres_per_mm = f64::from(georef.scale_denominator) / 1000.0;
    let half_widths: HashMap<u32, f64> = PATH_WIDTHS_MM
        .iter()
        .filter_map(|&(code, width)| {
            let half =
                width * kind.symbol_enlargement(georef.scale_denominator) * metres_per_mm / 2.0;
            Some((symbol_set.id_for_code(code)?, half))
        })
        .collect();
    let mut footprints: Vec<Polygon<f64>> = areas
        .iter()
        .chain(candidates)
        .filter(|o| paved_ids.contains(&o.symbol_id))
        .filter_map(object_polygon)
        .map(|p| projected(&p, georef))
        .collect();
    for object in candidates.iter().chain(map) {
        let Some(&half) = half_widths.get(&object.symbol_id) else {
            continue;
        };
        if !matches!(object.kind, MapObjectKind::Path) || object.coords.len() < 2 {
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
        footprints.extend(line.buffer(half).0);
    }
    footprints
}

/// The rings of `piece` outside `no_boundary`, joined where a ring was split at its start,
/// simplified by `tolerance` and at least `min_length` long (all in the map's CRS).
pub(crate) fn boundary_lines(
    piece: &Polygon<f64>,
    no_boundary: &MultiPolygon<f64>,
    min_length: f64,
    tolerance: f64,
) -> Vec<LineString<f64>> {
    let mut lines = Vec::new();
    for ring in std::iter::once(piece.exterior()).chain(piece.interiors()) {
        let rings = MultiLineString(vec![ring.clone()]);
        let kept = if no_boundary.0.is_empty() {
            rings
        } else {
            no_boundary.clip(&rings, true)
        };
        // The clip may split a ring that is kept across its start: join such pieces.
        let mut pieces: Vec<Vec<Coord<f64>>> = kept.0.into_iter().map(|l| l.0).collect();
        let mut joined = true;
        while joined && pieces.len() > 1 {
            joined = false;
            'outer: for a in 0..pieces.len() {
                for b in 0..pieces.len() {
                    if a != b && pieces[a].last() == pieces[b].first() {
                        let tail = pieces.remove(b);
                        let a = if b < a { a - 1 } else { a };
                        pieces[a].extend(tail.into_iter().skip(1));
                        joined = true;
                        break 'outer;
                    }
                }
            }
        }
        lines.extend(
            pieces
                .into_iter()
                .map(|p| LineString(p).simplify(tolerance))
                .filter(|l| Euclidean.length(l) >= min_length),
        );
    }
    lines
}

/// The map coordinates of `line` (map CRS); a closed line is closed as OOM expects.
pub(crate) fn line_coords(
    line: &LineString<f64>,
    georef: &Georeferencing,
) -> Result<Vec<MapCoord>, GeorefError> {
    let points = line
        .coords()
        .map(|c| {
            georef
                .projected_to_map(ProjectedPoint {
                    easting: c.x,
                    northing: c.y,
                })
                .map(|m| (m.x, m.y))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(if line.is_closed() && points.len() > 3 {
        closed_ring(&points)
    } else {
        points
            .into_iter()
            .map(|(x, y)| MapCoord::new(x, y))
            .collect()
    })
}

/// The trees `cluster` form a row: they lie in a corridor narrower than [`ROW_WIDTH_SIZES`]
/// symbol sizes, or the cluster is [`is_elongated`] and narrower than [`ROW_MAX_WIDTH_CROWNS`]
/// mean crowns, all along its main axis (the principal axis of the tree centres). `metre`: map
/// coordinates per ground metre.
fn is_row(cluster: &[usize], spots: &[Spot], metre: f64) -> bool {
    let (length, width, size, crown) = extent(cluster, spots);
    width < ROW_WIDTH_SIZES * size
        || (is_elongated(length, width, size) && width < ROW_MAX_WIDTH_CROWNS * crown * metre)
}

/// A cluster `length` long and `width` wide (along and across its main axis) of trees of symbol
/// size `size` is at least [`ROW_ASPECT`] times as long as wide, one symbol size added to both.
fn is_elongated(length: f64, width: f64, size: f64) -> bool {
    length + size >= ROW_ASPECT * (width + size)
}

/// The length and width of the trees `cluster` along and across its main axis, the mean symbol
/// size (map coordinates) and the mean crown (ground metres).
fn extent(cluster: &[usize], spots: &[Spot]) -> (f64, f64, f64, f64) {
    let n = cluster.len() as f64;
    let mean = |f: &dyn Fn(&Spot) -> f64| cluster.iter().map(|&i| f(&spots[i])).sum::<f64>() / n;
    let (mx, my) = (mean(&|s| s.xy.0), mean(&|s| s.xy.1));
    let sxx = mean(&|s| (s.xy.0 - mx).powi(2));
    let syy = mean(&|s| (s.xy.1 - my).powi(2));
    let sxy = mean(&|s| (s.xy.0 - mx) * (s.xy.1 - my));
    let angle = 0.5 * (2.0 * sxy).atan2(sxx - syy);
    let (cos, sin) = (angle.cos(), angle.sin());
    let (mut along, mut across) = ((f64::MAX, f64::MIN), (f64::MAX, f64::MIN));
    for &i in cluster {
        let (x, y) = (spots[i].xy.0 - mx, spots[i].xy.1 - my);
        let (u, v) = (x * cos + y * sin, -x * sin + y * cos);
        along = (along.0.min(u), along.1.max(u));
        across = (across.0.min(v), across.1.max(v));
    }
    (
        along.1 - along.0,
        across.1 - across.0,
        mean(&|s| s.size),
        mean(&|s| s.crown),
    )
}

/// The clusters of at least [`GROUP_MIN_TREES`] among the `trees` (indices into `spots`): single
/// linkage, two trees are in one cluster when they are closer than their minimum distance or
/// than `spacing`, or their crowns come closer than `close` (map coordinates; `metre` map
/// coordinates per ground metre). Each cluster is sorted, the clusters by their first tree.
fn tree_clusters(
    spots: &[Spot],
    trees: &[usize],
    gap: f64,
    (close, spacing): (f64, f64),
    metre: f64,
) -> Vec<Vec<usize>> {
    let link = |a: &Spot, b: &Spot| {
        a.min_distance(b, gap)
            .max((a.crown + b.crown) / 2.0 * metre + close)
            .max(spacing)
    };
    let largest = trees.iter().map(|&i| spots[i].size).fold(0.0, f64::max);
    let crown = trees.iter().map(|&i| spots[i].crown).fold(0.0, f64::max);
    // A grid of cells of the largest pair distance: a close tree lies in a neighbouring cell.
    let cell = (largest + gap).max(crown * metre + close).max(spacing);
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
                    if spots[i].distance(other) < link(&spots[i], other) {
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
/// 1. a point whose centre lies in a 520 map object (among `map`) is removed; a candidate point
///    in a 520 candidate (`out_of_bounds`, or one among `candidates`) stays a candidate tagged
///    [`HIDDEN_KEY`]` = 520` and takes no part in the rest: the training map leaves it out;
/// 2. at least [`GROUP_MIN_TREES`] candidate trees that keep less than the minimum gap from one
///    another or whose crowns come closer than [`GROUP_CLOSE_MM`], and are no row ([`is_row`]),
///    become a group area (`group_areas`) where it reaches the symbol's minimum: 501.3 on
///    paving, else 405 or 402, a 405 off open land with a 416 boundary;
/// 3. every other point keeps [`POINT_GAP_MM`] (× the enlargement) between the outer edges of
///    the symbols ([`point_size_mm`]); the smaller of two points goes ([`larger_first`]), and
///    the kept one names it in `freemapper:generalized` (a map point names no candidate). A
///    candidate never drops a map point.
///
/// `paved` are the drawn paved areas, `open_land` more areas that may be open land (401 to 404
/// count, as among `candidates` and `map`), `footprints` the 521 buildings and `water_union` the
/// drawn water in the map's CRS; `clip` is the map area plus buffer. Returns the kept candidates
/// and map objects in their order and the new group areas.
#[allow(clippy::too_many_arguments)]
pub fn generalize_points(
    candidates: Vec<MapObject>,
    map: Vec<MapObject>,
    out_of_bounds: &[MapObject],
    paved: &[MapObject],
    open_land: &[MapObject],
    footprints: &[Polygon<f64>],
    water_union: &MultiPolygon<f64>,
    clip: &BoundingBox,
    kind: MapKind,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<GeneralizedPoints, GeorefError> {
    let is_point = |o: &MapObject| matches!(o.kind, MapObjectKind::Point { .. });
    // 1. A 520 map object removes the points in it; a 520 candidate only hides the candidate
    //    points (they stay in the candidates, marked, and take no part below).
    let fence_id = symbol_set.id_for_code(OUT_OF_BOUNDS_CODE);
    let metre = 1e6 / f64::from(georef.scale_denominator);
    let fences = |objects: &mut dyn Iterator<Item = &MapObject>| {
        let polygons: Vec<Polygon<f64>> = objects
            .filter(|o| Some(o.symbol_id) == fence_id)
            .filter_map(object_polygon)
            .collect();
        Indexed::new(polygons, GRID_CELL_M * metre)
    };
    let map_fences = fences(&mut map.iter());
    let candidate_fences = fences(&mut out_of_bounds.iter().chain(&candidates));
    let inside = |fences: &Indexed, o: &MapObject| {
        is_point(o)
            && o.coords
                .first()
                .is_some_and(|c| fences.contains(Point::new(f64::from(c.x), f64::from(c.y))))
    };
    let map: Vec<MapObject> = map
        .into_iter()
        .filter(|o| !inside(&map_fences, o))
        .collect();
    let candidates: Vec<MapObject> = candidates
        .into_iter()
        .filter(|o| !inside(&map_fences, o))
        .map(|mut o| {
            if inside(&candidate_fences, &o) {
                o.tags
                    .push((HIDDEN_KEY.to_string(), OUT_OF_BOUNDS_CODE.to_string()));
            }
            o
        })
        .collect();

    // The points with a size, map objects first.
    let sizes: HashMap<u32, (&'static str, f64)> = POINT_SIZES_MM
        .iter()
        .filter_map(|&(code, _, _)| {
            Some((
                symbol_set.id_for_code(code)?,
                (
                    code,
                    point_size_mm(code, kind, georef.scale_denominator)? * 1000.0,
                ),
            ))
        })
        .collect();
    // On a sprint map a candidate point whose centre lies in a building cannot be seen or used:
    // it goes (ADR 0076). Trees stay: the cadastre footprint of a building often takes in the
    // trees at its wall, and a group of them is cut at the building later. Map points and the
    // schoolyard map keep theirs.
    let candidates: Vec<MapObject> =
        if matches!(kind, MapKind::Sprint | MapKind::Forest) && !footprints.is_empty() {
            let buildings = Indexed::new(footprints.to_vec(), GRID_CELL_M);
            candidates
                .into_iter()
                .filter(|o| {
                    let inside = is_point(o)
                        && sizes
                            .get(&o.symbol_id)
                            .is_some_and(|(code, _)| !TREE_CODES.contains(code))
                        && o.coords.first().is_some_and(|c| {
                            let p = georef.map_to_projected(*c);
                            buildings.contains(Point::new(p.easting, p.northing))
                        });
                    !inside
                })
                .collect()
        } else {
            candidates
        };
    // Control sites (ADR 0074, 0077): the ones of OSM, the street survey and the lighting
    // cadastre are thinned together, once the duplicates of the sources are gone and the sites
    // in buildings with them, so a site in a building does not take a neighbour out.
    let candidates = if matches!(kind, MapKind::Sprint | MapKind::Forest) {
        drop_ambiguous_control_sites(candidates, georef.scale_denominator)
    } else {
        candidates
    };
    let spot = |on_map: bool, index: usize, o: &MapObject| -> Option<Spot> {
        let &(code, size) = sizes
            .get(&o.symbol_id)
            .filter(|_| is_point(o) && tag(o, HIDDEN_KEY).is_none())?;
        let c = o.coords.first()?;
        let measured = TREE_CODES
            .contains(&code)
            .then(|| tag(o, "freemapper:crown_m"))
            .flatten()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|c| c.is_finite() && *c > 0.0);
        let crown = measured
            .or_else(|| {
                DEFAULT_CROWN_M
                    .iter()
                    .find(|(c, _)| *c == code)
                    .map(|(_, m)| *m)
            })
            .unwrap_or(0.0);
        Some(Spot {
            on_map,
            index,
            xy: (f64::from(c.x), f64::from(c.y)),
            size,
            code,
            crown,
            crown_measured: measured.is_some(),
            weak: tag(o, CONTROL_KEY).is_some(),
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
    let enlarge = kind.symbol_enlargement(georef.scale_denominator);
    let gap = POINT_GAP_MM * enlarge * 1000.0;

    // 2. Dense groups of candidate trees become group areas.
    let trees: Vec<usize> = (0..spots.len())
        .filter(|&i| !spots[i].on_map && spots[i].is_tree())
        .collect();
    let mut merged = vec![false; spots.len()];
    let mut areas = Vec::new();
    // A row (an avenue) stays points and is thinned.
    let close = GROUP_CLOSE_MM * enlarge * 1000.0;
    let spacing = GROUP_LINK_MM * enlarge * 1000.0;
    let clusters: Vec<Vec<usize>> = tree_clusters(&spots, &trees, gap, (close, spacing), metre)
        .into_iter()
        .filter(|c| !is_row(c, &spots, metre))
        .collect();
    if !clusters.is_empty() {
        let mut obstacles: Vec<Polygon<f64>> = map_fences
            .polygons
            .iter()
            .chain(&candidate_fences.polygons)
            .map(|f| projected(f, georef))
            .collect();
        obstacles.extend(footprints.iter().cloned());
        obstacles.extend(water_union.0.iter().cloned());
        let open_ids: Vec<u32> = OPEN_LAND_CODES
            .iter()
            .filter_map(|code| symbol_set.id_for_code(code))
            .collect();
        let ways = way_footprints(open_land, &candidates, &map, kind, georef, symbol_set);
        let open_land: Vec<Polygon<f64>> = open_land
            .iter()
            .chain(&candidates)
            .chain(&map)
            .filter(|o| open_ids.contains(&o.symbol_id))
            .filter_map(object_polygon)
            .map(|p| projected(&p, georef))
            .collect();
        let context = GroupContext {
            obstacles: Indexed::new(obstacles, GRID_CELL_M),
            ways: Indexed::new(ways, GRID_CELL_M),
            white_base: symbol_set
                .id_for_code(crate::omap::symbol_set::WALKABLE_AREA_CODE)
                .is_none(),
            paved: Indexed::new(
                paved
                    .iter()
                    .filter_map(object_polygon)
                    .map(|p| projected(&p, georef))
                    .collect(),
                GRID_CELL_M,
            ),
            open_land: Indexed::new(open_land, GRID_CELL_M),
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
                // The map part names no candidate.
                if s.on_map || !spots[j].on_map {
                    generalized[j].push(&s.name);
                }
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
            set: kind.symbol_set(kind.default_scale()),
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

        /// The fixture without the walkable area: the base is white, as before ADR 0069.
        pub(crate) fn with_white_base(mut self) -> Self {
            self.set = crate::omap::SymbolSet::issprom_2019().with_sheet_symbols();
            self
        }

        /// [`generalize_points`] with these inputs, no water and no other open land.
        pub(crate) fn run(
            &self,
            candidates: Vec<MapObject>,
            map: Vec<MapObject>,
            out_of_bounds: &[MapObject],
            paved: &[MapObject],
            footprints: &[Polygon<f64>],
        ) -> GeneralizedPoints {
            self.run_with(candidates, map, out_of_bounds, paved, &[], footprints)
        }

        /// [`Fixture::run`] on open land: a 401 candidate of 400 × 400 m round the reference
        /// point among the other open land.
        pub(crate) fn run_open(
            &self,
            candidates: Vec<MapObject>,
            map: Vec<MapObject>,
            out_of_bounds: &[MapObject],
            paved: &[MapObject],
            footprints: &[Polygon<f64>],
        ) -> GeneralizedPoints {
            let land = self.rect("401", (-200.0, -200.0), (400.0, 400.0));
            self.run_with(candidates, map, out_of_bounds, paved, &[land], footprints)
        }

        /// [`generalize_points`] with these inputs and `open_land`, no water.
        pub(crate) fn run_with(
            &self,
            candidates: Vec<MapObject>,
            map: Vec<MapObject>,
            out_of_bounds: &[MapObject],
            paved: &[MapObject],
            open_land: &[MapObject],
            footprints: &[Polygon<f64>],
        ) -> GeneralizedPoints {
            generalize_points(
                candidates,
                map,
                out_of_bounds,
                paved,
                open_land,
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
        let polygon = crate::build::geometry::object_polygon(object).expect("an area");
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

    /// The ids of the candidates hidden under a 520 candidate.
    fn hidden(objects: &[MapObject]) -> Vec<&str> {
        objects
            .iter()
            .filter(|o| tag(o, HIDDEN_KEY) == Some(OUT_OF_BOUNDS_CODE))
            .map(|o| tag(o, "freemapper:source_id").unwrap_or(""))
            .collect()
    }

    #[test]
    fn trees_on_out_of_bounds_are_hidden() {
        let f = fixture(MapKind::Sprint);
        let fence = f.rect("520", (0.0, 0.0), (20.0, 20.0));
        let inside = || f.tree("418", REGISTER, "in", (1.0, 10.0), None);
        let outside = || f.tree("418", REGISTER, "out", (-0.5, 10.0), None);
        // Under a 520 candidate the tree stays a candidate, marked hidden (left out of the
        // training map); it drops no visible point, and none drops it.
        let found = f.run(vec![fence.clone(), inside()], vec![], &[], &[], &[]);
        assert_eq!(ids(&found.candidates), ["", "in"]);
        assert_eq!(hidden(&found.candidates), ["in"]);
        let found = f.run(
            vec![inside(), outside()],
            vec![],
            std::slice::from_ref(&fence),
            &[],
            &[],
        );
        assert_eq!(ids(&found.candidates), ["in", "out"]);
        assert_eq!(hidden(&found.candidates), ["in"]);
        assert!(
            found
                .candidates
                .iter()
                .all(|o| tag(o, "freemapper:generalized").is_none())
        );
        // A 520 map object removes it.
        let found = f.run(vec![inside()], vec![fence.clone()], &[], &[], &[]);
        assert!(found.candidates.is_empty());
        assert_eq!(found.map, [fence]);
    }

    #[test]
    fn map_point_not_hidden_by_candidate_520() {
        let f = fixture(MapKind::Sprint);
        let fence = f.rect("520", (0.0, 0.0), (20.0, 20.0));
        let tree = f.tree("418", "osm", "node/1", (1.0, 10.0), None);
        let found = f.run(
            vec![fence.clone()],
            vec![tree.clone()],
            std::slice::from_ref(&fence),
            &[],
            &[],
        );
        assert_eq!(found.map, std::slice::from_ref(&tree));
        // Under a 520 map object it goes.
        let found = f.run(vec![], vec![fence.clone(), tree], &[], &[], &[]);
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
        assert_eq!(ids(&found.candidates), ["node/1", "node/2"]);
        assert_eq!(hidden(&found.candidates), ["node/1"]);
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
        let found = f.run_open(trees, vec![], &[], &[], &[]);
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

    /// The 416 boundaries among `areas`.
    fn boundaries<'a>(f: &Fixture, areas: &'a [MapObject]) -> Vec<&'a MapObject> {
        areas
            .iter()
            .filter(|o| o.symbol_id == f.id("416"))
            .collect()
    }

    /// The group areas among `areas` (no 416).
    fn group_areas_of<'a>(f: &Fixture, areas: &'a [MapObject]) -> Vec<&'a MapObject> {
        areas
            .iter()
            .filter(|o| o.symbol_id != f.id("416"))
            .collect()
    }

    #[test]
    fn park_group_on_white_background_becomes_bounded_forest() {
        // On the walkable area (ADR 0069) the brown base shows the white 405: no 416.
        let f = fixture(MapKind::Sprint);
        let found = f.run(f.grid(REGISTER, (3, 3), 3.5, 6.0), vec![], &[], &[], &[]);
        assert_eq!(group_areas_of(&f, &found.areas).len(), 1);
        assert!(boundaries(&f, &found.areas).is_empty(), "{:?}", found.areas);
        let f = fixture(MapKind::Sprint).with_white_base();
        // The dense group of `dense_group_becomes_forest_area` without open land under it: the
        // white 405 alone would not show, so a 416 boundary outlines it (user ruling of
        // 2026-11-10, superseding the open-land-only ruling).
        let found = f.run(f.grid(REGISTER, (3, 3), 3.5, 6.0), vec![], &[], &[], &[]);
        assert!(found.candidates.is_empty(), "{:?}", found.candidates);
        let areas = group_areas_of(&f, &found.areas);
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0].symbol_id, f.id("405"));
        let lines = boundaries(&f, &found.areas);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert_eq!(lines[0].kind, MapObjectKind::Path);
        // The boundary carries the area's tags: the merged trees and the generalization.
        assert_eq!(lines[0].tags, areas[0].tags);
        assert_eq!(
            tag(lines[0], "freemapper:source"),
            Some(GENERALIZATION_SOURCE)
        );
        // A closed ring round the area.
        assert!(lines[0].coords.len() > 4);
        assert_eq!(
            (lines[0].coords[0].x, lines[0].coords[0].y),
            (
                lines[0].coords.last().unwrap().x,
                lines[0].coords.last().unwrap().y
            )
        );
    }

    #[test]
    fn sparse_group_on_white_background_becomes_scattered_trees() {
        let f = fixture(MapKind::Sprint);
        // Small crowns closer than the symbols allow: a 402, whose yellow shows on its own.
        let found = f.run(f.grid(REGISTER, (5, 4), 3.5, 2.0), vec![], &[], &[], &[]);
        assert!(found.candidates.is_empty());
        assert_eq!(found.areas.len(), 1, "{:?}", found.areas);
        assert_eq!(found.areas[0].symbol_id, f.id("402"));
    }

    #[test]
    fn crowns_that_touch_make_a_group() {
        let f = fixture(MapKind::Sprint);
        // 16 trees 6 m apart: the symbols keep their gap (3.8 m), so before the groups they all
        // stayed points.
        let block = |crown: Option<f64>| -> Vec<MapObject> {
            (0..16)
                .map(|k| {
                    let at = (f64::from(k % 4) * 6.0, f64::from(k / 4) * 6.0);
                    f.tree("418", "trees/potsdam", &format!("p{k:02}"), at, crown)
                })
                .collect()
        };
        // Default 5 m crowns cover less than half of the group: a 402.
        let found = f.run(block(None), vec![], &[], &[], &[]);
        assert!(found.candidates.is_empty(), "{:?}", found.candidates);
        assert_eq!(found.areas.len(), 1, "{:?}", found.areas);
        assert_eq!(found.areas[0].symbol_id, f.id("402"));
        // 8 m crowns overlap: a closed canopy, forest; the walkable area shows it, no 416.
        let found = f.run(block(Some(8.0)), vec![], &[], &[], &[]);
        assert!(found.candidates.is_empty(), "{:?}", found.candidates);
        let areas = group_areas_of(&f, &found.areas);
        assert_eq!(areas.len(), 1, "{areas:?}");
        assert_eq!(areas[0].symbol_id, f.id("405"));
        assert!(boundaries(&f, &found.areas).is_empty());
    }

    #[test]
    fn sparse_trees_stay_points() {
        let f = fixture(MapKind::Sprint);
        // Default 5 m crowns 10 m apart: 5 m of lawn between them, no group.
        let trees: Vec<MapObject> = (0..16)
            .map(|k| {
                let at = (f64::from(k % 4) * 10.0, f64::from(k / 4) * 10.0);
                f.tree("418", "trees/potsdam", &format!("p{k:02}"), at, None)
            })
            .collect();
        let found = f.run(trees.clone(), vec![], &[], &[], &[]);
        assert!(found.areas.is_empty(), "{:?}", found.areas);
        assert_eq!(found.candidates, trees);
    }

    #[test]
    fn street_rows_stay_points_and_park_block_becomes_forest() {
        let f = fixture(MapKind::Sprint);
        // A Berlin street: large trees (crown 12 m, 417) 9 m apart on both sides of a 12 m
        // road, crowns meeting over it. Their rows stay points.
        let mut trees = Vec::new();
        for side in [0.0, 12.0] {
            for i in 0..12 {
                let id = format!("s{side}-{i:02}");
                trees.push(f.tree("417", REGISTER, &id, (f64::from(i) * 9.0, side), Some(12.0)));
            }
        }
        // A park block 60 m north: 417 (crown 14 m) and 418 (crown 6 m) mixed, 7 m apart.
        for k in 0..36 {
            let (i, j) = (f64::from(k % 6), f64::from(k / 6));
            let (code, crown) = if k % 3 == 0 {
                ("417", 14.0)
            } else {
                ("418", 6.0)
            };
            let id = format!("b{k:02}");
            trees.push(f.tree(code, REGISTER, &id, (i * 7.0, 60.0 + j * 7.0), Some(crown)));
        }
        let found = f.run(trees, vec![], &[], &[], &[]);
        let areas = group_areas_of(&f, &found.areas);
        assert_eq!(areas.len(), 1, "{areas:?}");
        assert_eq!(areas[0].symbol_id, f.id("405"));
        let merged = tag(areas[0], "freemapper:source_id").unwrap();
        assert_eq!(merged.split("; ").count(), 36, "{merged}");
        assert!(!merged.contains("/s"), "{merged}");
        assert!(boundaries(&f, &found.areas).is_empty());
        // The street trees stay points, thinned to the gap (417 to 417: 6.2 m).
        let left = ids(&found.candidates);
        assert!(left.iter().all(|id| id.starts_with('s')), "{left:?}");
        assert_eq!(left.len(), 24, "{left:?}");
    }

    #[test]
    fn forest_is_cut_at_a_road_and_has_no_boundary_there() {
        let f = fixture(MapKind::Sprint);
        // A 6 × 6 block of 418 (crown 6 m, 4 m apart) with a paved road across its middle.
        let trees = f.grid(REGISTER, (6, 6), 4.0, 6.0);
        let road = f.rect("501.5", (-10.0, 8.5), (40.0, 4.0));
        let found = f.run(trees, vec![], &[], std::slice::from_ref(&road), &[]);
        let road = projected(&f, &road);
        let areas = group_areas_of(&f, &found.areas);
        assert_eq!(areas.len(), 2, "{areas:?}");
        for area in &areas {
            assert_eq!(area.symbol_id, f.id("405"));
            assert!(projected(&f, area).intersection(&road).unsigned_area() < 0.01);
        }
        // No 416 runs along the road.
        use geo::Distance as _;
        for line in boundaries(&f, &found.areas) {
            let along = line
                .coords
                .iter()
                .map(|c| {
                    let p = f.georef.map_to_projected(*c);
                    geo_types::Point::new(p.easting, p.northing)
                })
                .filter(|p| geo::Euclidean.distance(p, &road) < 0.2)
                .count();
            assert!(along <= 2, "{along} points of a 416 on the road edge");
        }
    }

    #[test]
    fn group_on_a_pitch_becomes_area() {
        let f = fixture(MapKind::Sprint);
        let pitch = || f.rect("401", (-10.0, -10.0), (40.0, 40.0));
        // An OSM grass pitch among the candidates.
        let mut candidates = vec![pitch()];
        candidates.extend(f.grid(REGISTER, (3, 3), 3.5, 6.0));
        let found = f.run(candidates, vec![], &[], &[], &[]);
        assert_eq!(found.areas.len(), 1);
        assert_eq!(found.areas[0].symbol_id, f.id("405"));
        // On open land the yellow round the 405 shows it: no 416.
        assert!(boundaries(&f, &found.areas).is_empty());
        assert_eq!(found.candidates, [pitch()]);
        // A green strip of the paved build, and a 401 map object.
        let trees = || f.grid(REGISTER, (5, 4), 3.5, 2.0);
        let found = f.run_with(trees(), vec![], &[], &[], &[pitch()], &[]);
        assert_eq!(found.areas.len(), 1);
        assert_eq!(found.areas[0].symbol_id, f.id("402"));
        let found = f.run(trees(), vec![pitch()], &[], &[], &[]);
        assert_eq!(found.areas.len(), 1);
        // A forest with less than half of its closed crowns on open land lies on the white base
        // (a map without the walkable area): it gets the 416 boundary.
        let strip = || f.rect("401", (-10.0, -10.0), (40.0, 13.0));
        let forest = || f.grid(REGISTER, (3, 3), 3.5, 6.0);
        let white = fixture(MapKind::Sprint).with_white_base();
        let found = white.run_with(forest(), vec![], &[], &[], &[strip()], &[]);
        assert_eq!(group_areas_of(&white, &found.areas).len(), 1);
        assert_eq!(boundaries(&white, &found.areas).len(), 1);
        // On the walkable area it needs none.
        let found = f.run_with(forest(), vec![], &[], &[], &[strip()], &[]);
        assert_eq!(group_areas_of(&f, &found.areas).len(), 1);
        assert!(boundaries(&f, &found.areas).is_empty());
    }

    #[test]
    fn schoolyard_forest_off_open_land_keeps_its_boundary() {
        // The school map has no walkable area: a 405 on its white base needs the 416.
        let f = fixture(MapKind::Schoolyard);
        let found = f.run(f.grid(REGISTER, (3, 3), 3.0, 5.0), vec![], &[], &[], &[]);
        assert_eq!(
            group_areas_of(&f, &found.areas).len(),
            1,
            "{:?}",
            found.areas
        );
        assert_eq!(boundaries(&f, &found.areas).len(), 1);
    }

    /// The footprint of a 505.1 path candidate from `a` to `b` (metres from the reference point).
    fn path(f: &Fixture, code: &str, a: (f64, f64), b: (f64, f64)) -> MapObject {
        MapObject {
            symbol_id: f.id(code),
            kind: MapObjectKind::Path,
            coords: vec![f.at(a), f.at(b)],
            tags: vec![("freemapper:source".into(), "osm".into())],
        }
    }

    #[test]
    fn forest_never_covers_a_path_or_a_paved_candidate() {
        let f = fixture(MapKind::Sprint);
        // An 8 × 8 block of 418 (crown 6 m, 4 m apart) on a park, a path candidate across its
        // middle and a paved candidate (the survey's paving) along its south edge.
        let land = f.rect("401", (-200.0, -200.0), (400.0, 400.0));
        let paved = f.rect("501.5", (-10.0, -4.0), (50.0, 3.0));
        for code in ["505.1", "506"] {
            let way = path(&f, code, (-10.0, 14.0), (40.0, 14.0));
            let mut candidates = f.grid(REGISTER, (8, 8), 4.0, 6.0);
            candidates.push(way.clone());
            let found = f.run_with(
                candidates,
                vec![],
                &[],
                &[],
                &[land.clone(), paved.clone()],
                &[],
            );
            let areas = group_areas_of(&f, &found.areas);
            assert_eq!(areas.len(), 2, "{code}: {areas:?}");
            let line = geo_types::LineString::from(
                way.coords
                    .iter()
                    .map(|c| {
                        let p = f.georef.map_to_projected(*c);
                        (p.easting, p.northing)
                    })
                    .collect::<Vec<_>>(),
            );
            let paved = projected(&f, &paved);
            for area in &areas {
                assert_eq!(area.symbol_id, f.id("405"));
                let area = projected(&f, area);
                assert!(
                    !area.intersects(&line),
                    "{code}: the forest covers the path"
                );
                assert!(area.intersection(&paved).unsigned_area() < 0.01);
            }
            assert!(boundaries(&f, &found.areas).is_empty());
            // The path stays among the candidates.
            assert!(found.candidates.contains(&way));
        }
    }

    #[test]
    fn forest_never_covers_a_firm_gravel_path_map_object() {
        // ADR 0079: the 501.6 to 501.9 ways are map objects now; a tree group is still cut at
        // them, each by its own width on paper.
        let f = fixture(MapKind::Sprint);
        let land = f.rect("401", (-200.0, -200.0), (400.0, 400.0));
        for code in ["501.6", "501.7", "501.8", "501.9"] {
            let way = path(&f, code, (-10.0, 14.0), (40.0, 14.0));
            let candidates = f.grid(REGISTER, (8, 8), 4.0, 6.0);
            let found = f.run_with(
                candidates,
                vec![way.clone()],
                &[],
                &[],
                std::slice::from_ref(&land),
                &[],
            );
            let areas = group_areas_of(&f, &found.areas);
            assert!(!areas.is_empty(), "{code}");
            let line = geo_types::LineString::from(
                way.coords
                    .iter()
                    .map(|c| {
                        let p = f.georef.map_to_projected(*c);
                        (p.easting, p.northing)
                    })
                    .collect::<Vec<_>>(),
            );
            for area in &areas {
                assert!(
                    !projected(&f, area).intersects(&line),
                    "{code}: the forest covers the path"
                );
            }
            assert!(
                found.map.contains(&way),
                "{code}: the path stays on the map"
            );
        }
    }

    #[test]
    fn forest_never_covers_a_railway_map_object() {
        // ADR 0093: a 509.1 railway is a map object like the firm-gravel ways; a tree group is
        // cut at it.
        let f = fixture(MapKind::Sprint);
        let land = f.rect("401", (-200.0, -200.0), (400.0, 400.0));
        let way = path(&f, "509.1", (-10.0, 14.0), (40.0, 14.0));
        let found = f.run_with(
            f.grid(REGISTER, (8, 8), 4.0, 6.0),
            vec![way.clone()],
            &[],
            &[],
            std::slice::from_ref(&land),
            &[],
        );
        let areas = group_areas_of(&f, &found.areas);
        assert!(!areas.is_empty());
        let line = geo_types::LineString::from(
            way.coords
                .iter()
                .map(|c| {
                    let p = f.georef.map_to_projected(*c);
                    (p.easting, p.northing)
                })
                .collect::<Vec<_>>(),
        );
        for area in &areas {
            assert!(
                !projected(&f, area).intersects(&line),
                "the forest covers the railway"
            );
        }
        assert!(found.map.contains(&way));
    }

    #[test]
    fn sparse_group_becomes_scattered_trees_area() {
        let f = fixture(MapKind::Sprint);
        let found = f.run_open(f.grid(REGISTER, (5, 4), 3.5, 2.0), vec![], &[], &[], &[]);
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
        let found = f.run_open(trees, vec![], &[], &[], &[]);
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
        let found = f.run_open(
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
        let found = f.run_open(
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
        // The trees under the building stay points, thinned; the two in the 520 stay hidden.
        assert_eq!(hidden(&found.candidates), ["t03", "t04"]);
        let left: Vec<(f64, f64)> = found
            .candidates
            .iter()
            .filter(|o| tag(o, HIDDEN_KEY).is_none())
            .map(|o| {
                let p = f.georef.map_to_projected(o.coords[0]);
                (p.easting, p.northing)
            })
            .collect();
        use geo::Contains as _;
        assert!(!left.is_empty());
        assert!(
            left.iter()
                .all(|&(e, n)| !fence.contains(&geo_types::Point::new(e, n)))
        );
        // No tree under the building (the columns at 10.5 and 14 m, t15 to t24) is merged; they
        // stay points, thinned (3.5 m apart).
        for area in &found.areas {
            let merged = tag(area, "freemapper:source_id").unwrap();
            assert!(
                (15..25).all(|k| !merged.contains(&format!("t{k:02}"))),
                "{merged}"
            );
        }
        let under = left
            .iter()
            .filter(|&&(e, n)| building.contains(&geo_types::Point::new(e, n)))
            .count();
        assert!((1..10).contains(&under), "{left:?}");
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
    fn candidate_points_inside_a_building_are_dropped_on_sprint_maps() {
        // A memorial, a lamp or a fountain whose centre lies in a 521 footprint (the dots on the
        // church of the Lübeck example) cannot be seen or used on the map (ADR 0076). A tree stays.
        let f = fixture(MapKind::Sprint);
        let church = [f.projected_rect((0.0, 0.0), (40.0, 20.0))];
        let candidates = vec![
            f.point("526", "osm", "memorial", (10.0, 10.0), &[]),
            f.point(
                "531",
                "osm",
                "lamp",
                (20.0, 10.0),
                &[("freemapper:control", "street_lamp")],
            ),
            f.tree("418", "osm", "tree", (30.0, 10.0), None),
            f.point("311", "osm", "fountain", (35.0, 5.0), &[]),
            f.point("526", "osm", "outside", (50.0, 10.0), &[]),
            f.point("526", "osm", "edge", (39.0, 25.0), &[]),
        ];
        let found = f.run(candidates.clone(), vec![], &[], &[], &church);
        assert_eq!(ids(&found.candidates), ["tree", "outside", "edge"]);
        // Map points stay, and the schoolyard keeps what it had.
        let on_map = f.run(vec![], candidates, &[], &[], &church);
        assert_eq!(on_map.map.len(), 6);
        let school = fixture(MapKind::Schoolyard);
        let church = [school.projected_rect((0.0, 0.0), (40.0, 20.0))];
        let found = school.run(
            vec![school.point("545", "osm", "lamp", (20.0, 10.0), &[])],
            vec![],
            &[],
            &[],
            &church,
        );
        assert_eq!(ids(&found.candidates), ["lamp"]);
    }

    #[test]
    fn control_site_gives_way_to_every_other_point() {
        // A control site (ADR 0074) is the largest point symbol, but it must not knock out a
        // tree or a fountain that stands next to it.
        let f = fixture(MapKind::Sprint);
        let site = |id: &str, kind: &str, at: (f64, f64)| {
            f.point("531", "osm", id, at, &[("freemapper:control", kind)])
        };
        let found = f.run(
            vec![
                f.tree("418", REGISTER, "tree", (0.0, 0.0), None),
                site("node/1", "street_lamp", (3.0, 0.0)),
                f.point("311", "osm", "node/2", (0.0, 20.0), &[]),
                site("node/3", "bench", (3.0, 20.0)),
            ],
            vec![],
            &[],
            &[],
            &[],
        );
        assert_eq!(ids(&found.candidates), ["tree", "node/2"]);
        // Alone, or only beside a site of another kind, it stays; the first by name wins.
        let found = f.run(
            vec![
                site("node/9", "bench", (0.0, 0.0)),
                site("node/8", "street_lamp", (3.0, 0.0)),
            ],
            vec![],
            &[],
            &[],
            &[],
        );
        assert_eq!(ids(&found.candidates), ["node/8"]);
    }

    #[test]
    fn control_sites_of_one_kind_within_25_m_are_dropped_wherever_they_come_from() {
        // OSM, the street survey and the lighting cadastre are thinned together (ADR 0077),
        // after the buildings: a lamp in a building does not take its neighbour out.
        let f = fixture(MapKind::Sprint);
        let site = |source: &str, id: &str, at: (f64, f64)| {
            f.point(
                "531",
                source,
                id,
                at,
                &[("freemapper:control", "street_lamp")],
            )
        };
        let wall = [f.projected_rect((0.0, 0.0), (40.0, 4.0))];
        let found = f.run(
            vec![
                site("osm", "node/1", (0.0, 20.0)),
                site("geoportal-berlin/beleuchtung", "lamp/2", (20.0, 20.0)),
                site("osm", "node/3", (100.0, 20.0)),
                // In the building: gone, and it does not count for node/4.
                site("osm", "node/5", (100.0, 2.0)),
                site("osm", "node/4", (110.0, 20.0)),
                site("geoportal-berlin/beleuchtung", "lamp/6", (200.0, 20.0)),
            ],
            vec![],
            &[],
            &[],
            &wall,
        );
        // node/1 and lamp/2 are 20 m apart; node/3 and node/4 are 10 m apart.
        assert_eq!(ids(&found.candidates), ["lamp/6"]);
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
        // The map part names only the map points it dropped, no candidate.
        assert_eq!(
            tag(&found.map[0], "freemapper:generalized"),
            Some("osm/node/2")
        );
    }

    #[test]
    fn default_crown_does_not_beat_a_measured_one() {
        let f = fixture(MapKind::Sprint);
        let pair = |a: MapObject, b: MapObject| {
            let found = f.run(vec![a, b], vec![], &[], &[], &[]);
            ids(&found.candidates)
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>()
        };
        // An OSM tree without a crown (5 m by default) against a register tree of 4 m.
        assert_eq!(
            pair(
                f.tree("418", "osm", "node/1", (0.0, 0.0), None),
                f.tree("418", REGISTER, "measured", (3.0, 0.0), Some(4.0)),
            ),
            ["measured"]
        );
        // The same source: the measured crown wins over the default.
        assert_eq!(
            pair(
                f.tree("418", REGISTER, "a", (0.0, 0.0), None),
                f.tree("418", REGISTER, "b", (3.0, 0.0), Some(4.0)),
            ),
            ["b"]
        );
    }

    #[test]
    fn row_of_street_trees_stays_points() {
        let f = fixture(MapKind::Sprint);
        // An avenue of ten large trees 6 m apart, crowns 12 m: one closed crown strip.
        let mut trees: Vec<MapObject> = (0..10)
            .map(|i| {
                let id = format!("l{i}");
                f.tree("417", REGISTER, &id, (f64::from(i) * 6.0, 0.0), Some(12.0))
            })
            .collect();
        // A dense block 100 m away still becomes forest.
        let block: Vec<MapObject> = f
            .grid(REGISTER, (3, 3), 3.5, 6.0)
            .into_iter()
            .map(|mut t| {
                t.coords[0].y -= (100.0 * f.metre()) as i32;
                t
            })
            .collect();
        trees.extend(block);
        let found = f.run_open(trees, vec![], &[], &[], &[]);
        assert_eq!(found.areas.len(), 1);
        assert_eq!(found.areas[0].symbol_id, f.id("405"));
        assert!(
            !tag(&found.areas[0], "freemapper:source_id")
                .unwrap()
                .contains("/l")
        );
        // 417 to 417 needs 6.2 m: every other tree stays.
        assert_eq!(ids(&found.candidates), ["l0", "l2", "l4", "l6", "l8"]);
    }

    #[test]
    fn thin_tail_is_left_out_of_the_area() {
        let f = fixture(MapKind::Sprint);
        // A 402 block with a tail of eight trees in a line: the tail is one row wide.
        let mut trees = f.grid(REGISTER, (5, 4), 3.5, 2.0);
        trees.extend((0..8).map(|i| {
            let id = format!("u{i}");
            f.tree(
                "418",
                REGISTER,
                &id,
                (17.5 + f64::from(i) * 3.5, 3.5),
                Some(2.0),
            )
        }));
        let found = f.run_open(trees, vec![], &[], &[], &[]);
        assert_eq!(found.areas.len(), 1);
        let area = &found.areas[0];
        assert_eq!(area.symbol_id, f.id("402"));
        let merged = tag(area, "freemapper:source_id").unwrap();
        // The first tail trees lie in the block's own margin; the rest of the tail does not.
        assert!(
            (2..8).all(|k| !merged.contains(&format!("/u{k}"))),
            "{merged}"
        );
        // Nothing of the area reaches along the far tail.
        let tail = f.projected_rect((26.0, 0.0), (20.0, 7.0));
        assert!(projected(&f, area).intersection(&tail).unsigned_area() < 0.01);
        // The tail stays points, thinned to every other tree.
        assert_eq!(ids(&found.candidates), ["u2", "u4", "u6"]);
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
            let scale = kind.default_scale();
            let set = kind.symbol_set(scale);
            for (code, _, _) in POINT_SIZES_MM {
                if set.id_for_code(code).is_none() {
                    continue;
                }
                let size = point_size_mm(code, kind, scale).expect("in the table");
                let drawn = drawn_extent_mm(&set, code);
                // The enlarged school set rounds to whole micrometres.
                assert!(size >= drawn - 0.002, "{kind:?} {code}: {size} < {drawn}");
            }
        }
        // 417 at the norm's white mask, not the bundled 1.2 mm.
        assert!((point_size_mm("417", MapKind::Sprint, 4000).unwrap() - 1.4).abs() < 1e-9);
        assert!((point_size_mm("417", MapKind::Schoolyard, 1000).unwrap() - 1.75).abs() < 1e-9);
        assert!((point_size_mm("544", MapKind::Schoolyard, 1000).unwrap() - 1.1).abs() < 1e-9);
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
