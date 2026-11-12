//! Vegetation areas from a classified laser point cloud: open land, runnable forest, the green
//! shades of slow running, walk and impassable vegetation, and undergrowth (ISSprOM 401, 405 to
//! 410). For parks and, later, forest maps.
//!
//! Ideas from karttapullautin (open land as the share of low returns, majority filter, minimum
//! area) and omapmaker (height bands weighted by how much they slow a runner, the green index
//! (v − g) / (v + g), undergrowth as the share of returns just above the ground); the code is our
//! own. A surface model cannot do this: it shows the crowns, not whether one can run below them.
//! Everything here is a candidate (spec §4.5).

use geo::{Area as _, Contains as _, Simplify as _};

use super::DeriveError;
use super::contours::contour_lines;
use crate::raster::{GridSpec, Raster};

/// Cell of the classification.
pub const CELL_M: f64 = 3.0;
/// Fewer returns in a cell say nothing.
const MIN_RETURNS: usize = 4;
/// Below this height above ground a return is ground for the green index.
const GROUND_BELOW_M: f64 = 0.2;
/// Undergrowth band: knee to hip height.
const UNDERGROWTH_BELOW_M: f64 = 0.8;
/// Height bands of the green index and their weights: low bushes slow a runner most.
const BANDS: [(f64, f64, f64); 3] = [(0.8, 1.8, 1.0), (1.8, 3.0, 0.5), (3.0, f64::MAX, 0.1)];
/// Open land: at least this share of all returns lies lower than [`OPEN_BELOW_M`].
const OPEN_SHARE: f64 = 0.9;
const OPEN_BELOW_M: f64 = 0.9;
/// Green index (0 to 1) from which a cell is 406, 408, 410.
const SLOW_RUNNING_INDEX: f64 = 0.4;
const WALK_INDEX: f64 = 0.6;
const IMPASSABLE_INDEX: f64 = 0.8;
/// Undergrowth share (undergrowth / (undergrowth + ground)) from which runnable forest is 407, 409.
const UNDERGROWTH_SLOW: f64 = 0.35;
const UNDERGROWTH_WALK: f64 = 0.56;
/// Smallest area on paper (1.5 mm × 1.5 mm); smaller patches are left out, smaller holes filled.
const MIN_AREA_MM2: f64 = 2.25;
/// Outlines are simplified to this, on the ground.
const SIMPLIFY_M: f64 = 1.5;

/// One return of the point cloud.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VegetationPoint {
    pub e: f64,
    pub n: f64,
    /// Height above the terrain model (or above the ground points).
    pub height_m: f64,
    /// Classified as ground (LAS class 2).
    pub ground: bool,
}

/// Points read per chunk.
const CHUNK_POINTS: u64 = 1_000_000;

/// The returns of classified LAS/LAZ tiles over `dtm`, with their height above it. Buildings,
/// noise and water are left out, and so is every return over a cell of the DTM without data.
pub fn read_points(
    tiles: &[std::path::PathBuf],
    dtm: &Raster<f32>,
) -> Result<Vec<VegetationPoint>, DeriveError> {
    use las::point::Classification;
    let grid = dtm.grid;
    let las_error = |path: &std::path::Path, e: las::Error| {
        DeriveError::Image(format!("{}: {e}", path.display()))
    };
    let mut out = Vec::new();
    for path in tiles {
        let mut reader = las::Reader::from_path(path).map_err(|e| las_error(path, e))?;
        let mut points = las::PointDataBuilder::new()
            .for_header(reader.header())
            .build();
        while reader
            .fill_points(CHUNK_POINTS, &mut points)
            .map_err(|e| las_error(path, e))?
            > 0
        {
            for point in points.points() {
                let point = point.map_err(|e| las_error(path, e))?;
                if matches!(
                    point.classification,
                    Classification::Building
                        | Classification::LowPoint
                        | Classification::Water
                        | Classification::HighNoise
                ) {
                    continue;
                }
                let col = ((point.x - grid.west) / grid.pixel_size).floor();
                let row = ((grid.north - point.y) / grid.pixel_size).floor();
                if col < 0.0
                    || row < 0.0
                    || col >= f64::from(grid.width)
                    || row >= f64::from(grid.height)
                {
                    continue;
                }
                let ground = dtm.get(col as u32, row as u32);
                if ground.is_nan() {
                    continue;
                }
                out.push(VegetationPoint {
                    e: point.x,
                    n: point.y,
                    height_m: point.z - f64::from(ground),
                    ground: point.classification == Classification::Ground,
                });
            }
        }
    }
    Ok(out)
}

/// What a cell is, from open to impassable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VegetationClass {
    Open,
    Forest,
    SlowRunningGoodVisibility,
    WalkGoodVisibility,
    SlowRunning,
    Walk,
    Impassable,
}

impl VegetationClass {
    pub const ALL: [Self; 7] = [
        Self::Open,
        Self::Forest,
        Self::SlowRunningGoodVisibility,
        Self::WalkGoodVisibility,
        Self::SlowRunning,
        Self::Walk,
        Self::Impassable,
    ];

    /// The ISSprOM symbol.
    pub fn code(self) -> &'static str {
        match self {
            Self::Open => "401",
            Self::Forest => "405",
            Self::SlowRunningGoodVisibility => "407",
            Self::WalkGoodVisibility => "409",
            Self::SlowRunning => "406",
            Self::Walk => "408",
            Self::Impassable => "410",
        }
    }
}

/// Counts of one cell.
#[derive(Debug, Default, Clone, Copy)]
struct Counts {
    all: usize,
    low: usize,
    ground: f64,
    undergrowth: f64,
    vegetation: f64,
}

/// The class of every cell of `grid`; `None` where too few returns fall.
pub fn classify(
    points: impl IntoIterator<Item = VegetationPoint>,
    grid: GridSpec,
) -> Raster<Option<VegetationClass>> {
    let (w, h) = (grid.width as usize, grid.height as usize);
    let mut counts = vec![Counts::default(); w * h];
    for p in points {
        let x = ((p.e - grid.west) / grid.pixel_size).floor();
        let y = ((grid.north - p.n) / grid.pixel_size).floor();
        if x < 0.0 || y < 0.0 || x >= w as f64 || y >= h as f64 {
            continue;
        }
        let c = &mut counts[y as usize * w + x as usize];
        c.all += 1;
        if p.height_m < OPEN_BELOW_M || p.ground {
            c.low += 1;
        }
        if p.ground || p.height_m < GROUND_BELOW_M {
            c.ground += 1.0;
        } else if p.height_m < UNDERGROWTH_BELOW_M {
            c.undergrowth += 1.0;
        } else if let Some((_, _, weight)) = BANDS
            .iter()
            .find(|(lo, hi, _)| (*lo..*hi).contains(&p.height_m))
        {
            c.vegetation += weight;
        }
    }
    Raster {
        grid,
        data: counts.iter().map(cell_class).collect(),
    }
}

fn cell_class(c: &Counts) -> Option<VegetationClass> {
    if c.all < MIN_RETURNS {
        return None;
    }
    if c.low as f64 >= OPEN_SHARE * c.all as f64 {
        return Some(VegetationClass::Open);
    }
    let index = if c.vegetation + c.ground > 0.0 {
        ((c.vegetation - c.ground) / (c.vegetation + c.ground) + 1.0) / 2.0
    } else {
        0.0
    };
    let undergrowth = if c.undergrowth + c.ground > 0.0 {
        c.undergrowth / (c.undergrowth + c.ground)
    } else {
        0.0
    };
    Some(if index >= IMPASSABLE_INDEX {
        VegetationClass::Impassable
    } else if index >= WALK_INDEX {
        VegetationClass::Walk
    } else if index >= SLOW_RUNNING_INDEX {
        VegetationClass::SlowRunning
    } else if undergrowth >= UNDERGROWTH_WALK {
        VegetationClass::WalkGoodVisibility
    } else if undergrowth >= UNDERGROWTH_SLOW {
        VegetationClass::SlowRunningGoodVisibility
    } else {
        VegetationClass::Forest
    })
}

/// A cell that shares its class with at most one neighbour takes the class most of its 3 × 3
/// neighbourhood has: single cells and thin lines go, a block of four stays.
pub fn majority(classes: &Raster<Option<VegetationClass>>) -> Raster<Option<VegetationClass>> {
    let (w, h) = (classes.grid.width as i64, classes.grid.height as i64);
    let mut out = classes.clone();
    for y in 0..h {
        for x in 0..w {
            let own = classes.data[(y * w + x) as usize];
            let mut tally: Vec<(Option<VegetationClass>, usize)> = Vec::new();
            for (dx, dy) in (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (dx, dy))) {
                let (xx, yy) = (x + dx, y + dy);
                if xx < 0 || yy < 0 || xx >= w || yy >= h {
                    continue;
                }
                let class = classes.data[(yy * w + xx) as usize];
                match tally.iter_mut().find(|(c, _)| *c == class) {
                    Some(entry) => entry.1 += 1,
                    None => tally.push((class, 1)),
                }
            }
            let best = tally.iter().map(|(_, n)| *n).max().unwrap_or(0);
            let own_count = tally.iter().find(|(c, _)| *c == own).map_or(0, |(_, n)| *n);
            if own_count <= 2 && own_count < best {
                out.data[(y * w + x) as usize] = tally
                    .iter()
                    .filter(|(_, n)| *n == best)
                    .map(|(c, _)| *c)
                    .min()
                    .flatten();
            }
        }
    }
    out
}

/// The areas of every class, after the majority filter, simplified, without patches and holes
/// smaller than 1.5 mm × 1.5 mm on paper.
pub fn vegetation_areas(
    classes: &Raster<Option<VegetationClass>>,
    scale_denominator: u32,
) -> Vec<(VegetationClass, geo_types::MultiPolygon<f64>)> {
    let smoothed = majority(classes);
    let metres_per_mm = f64::from(scale_denominator) / 1000.0;
    let min_area_m2 = MIN_AREA_MM2 * metres_per_mm * metres_per_mm;
    VegetationClass::ALL
        .into_iter()
        .filter_map(|class| {
            let areas = outlines(&smoothed, class, min_area_m2);
            (!areas.0.is_empty()).then_some((class, areas))
        })
        .collect()
}

/// The cells of `class` as polygons: the 0.5 contour of a 0/1 raster padded by one empty cell,
/// so every ring closes; a ring inside an odd number of others is a hole.
fn outlines(
    classes: &Raster<Option<VegetationClass>>,
    class: VegetationClass,
    min_area_m2: f64,
) -> geo_types::MultiPolygon<f64> {
    let grid = classes.grid;
    let padded = GridSpec {
        west: grid.west - grid.pixel_size,
        north: grid.north + grid.pixel_size,
        pixel_size: grid.pixel_size,
        width: grid.width + 2,
        height: grid.height + 2,
    };
    let mut indicator = Raster::filled(padded, 0.25f32);
    let mut any = false;
    for y in 0..grid.height {
        for x in 0..grid.width {
            if classes.get(x, y) == Some(class) {
                indicator.set(x + 1, y + 1, 0.75);
                any = true;
            }
        }
    }
    if !any {
        return geo_types::MultiPolygon(Vec::new());
    }
    let rings: Vec<geo_types::Polygon<f64>> = contour_lines(&indicator, 0.5)
        .into_iter()
        .filter(|l| l.points.len() > 3 && l.points.first() == l.points.last())
        .map(|l| {
            let ring = geo_types::LineString::from(l.points).simplify(SIMPLIFY_M);
            geo_types::Polygon::new(ring, vec![])
        })
        .filter(|p| p.unsigned_area() >= min_area_m2)
        .collect();
    // Rings do not cross, so a ring lies inside another when one of its vertices does (an
    // interior point would not do: that of an outline can lie in its own hole).
    let vertex = |p: &geo_types::Polygon<f64>| geo_types::Point::from(p.exterior().0[0]);
    let depth = |i: usize| {
        let inside = vertex(&rings[i]);
        rings
            .iter()
            .enumerate()
            .filter(|(j, other)| *j != i && other.contains(&inside))
            .count()
    };
    let depths: Vec<usize> = (0..rings.len()).map(depth).collect();
    let mut polygons: Vec<geo_types::Polygon<f64>> = Vec::new();
    for (i, ring) in rings.iter().enumerate() {
        if depths[i].is_multiple_of(2) {
            let holes = rings
                .iter()
                .enumerate()
                .filter(|(j, hole)| depths[*j] == depths[i] + 1 && ring.contains(&vertex(hole)))
                .map(|(_, hole)| hole.exterior().clone())
                .collect();
            polygons.push(geo_types::Polygon::new(ring.exterior().clone(), holes));
        }
    }
    geo_types::MultiPolygon(polygons)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(cells: u32) -> GridSpec {
        GridSpec {
            west: 0.0,
            north: f64::from(cells) * CELL_M,
            pixel_size: CELL_M,
            width: cells,
            height: cells,
        }
    }

    /// `per_cell` returns at the given heights, spread over the cell at `(x, y)`.
    fn cell_points(x: u32, y: u32, heights: &[f64], g: &GridSpec) -> Vec<VegetationPoint> {
        heights
            .iter()
            .enumerate()
            .map(|(i, &h)| VegetationPoint {
                e: g.west + (f64::from(x) + 0.1 + 0.8 * (i as f64 / heights.len() as f64)) * CELL_M,
                n: g.north - (f64::from(y) + 0.5) * CELL_M,
                height_m: h,
                ground: h == 0.0,
            })
            .collect()
    }

    fn class_of(heights: &[f64]) -> Option<VegetationClass> {
        let g = grid(1);
        classify(cell_points(0, 0, heights, &g), g).get(0, 0)
    }

    #[test]
    fn a_lawn_is_open() {
        assert_eq!(class_of(&[0.0; 20]), Some(VegetationClass::Open));
        assert_eq!(
            class_of(&[0.1, 0.3, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0]),
            Some(VegetationClass::Open)
        );
    }

    #[test]
    fn tall_trees_over_a_clear_floor_are_runnable_forest() {
        let mut heights = vec![18.0; 10];
        heights.extend([0.0; 10]);
        assert_eq!(class_of(&heights), Some(VegetationClass::Forest));
    }

    #[test]
    fn dense_bushes_are_impassable() {
        let mut heights = vec![1.2; 18];
        heights.extend([0.0; 2]);
        assert_eq!(class_of(&heights), Some(VegetationClass::Impassable));
    }

    #[test]
    fn the_green_shades_grow_with_the_bushes() {
        // Ten ground returns and a growing number of returns at 1.2 m.
        let class = |bushes: usize| {
            let mut heights = vec![1.2; bushes];
            heights.extend([0.0; 10]);
            heights.extend([15.0; 4]);
            class_of(&heights).unwrap()
        };
        assert_eq!(class(3), VegetationClass::Forest);
        assert_eq!(class(10), VegetationClass::SlowRunning);
        assert_eq!(class(20), VegetationClass::Walk);
        assert_eq!(class(60), VegetationClass::Impassable);
    }

    #[test]
    fn undergrowth_below_trees_is_407_or_409() {
        let class = |undergrowth: usize| {
            let mut heights = vec![0.5; undergrowth];
            heights.extend([0.0; 10]);
            heights.extend([16.0; 10]);
            class_of(&heights).unwrap()
        };
        assert_eq!(class(6), VegetationClass::SlowRunningGoodVisibility);
        assert_eq!(class(15), VegetationClass::WalkGoodVisibility);
    }

    #[test]
    fn read_points_measures_above_the_dtm_and_skips_buildings() {
        use las::point::Classification;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tile.laz");
        let header = las::Builder::from((1, 2)).into_header().unwrap();
        let mut writer = las::Writer::from_path(&path, header).unwrap();
        for (x, z, classification) in [
            (1.0, 30.0, Classification::Ground),
            (2.0, 31.5, Classification::MediumVegetation),
            (4.0, 45.0, Classification::Building),
            (20.0, 31.0, Classification::LowVegetation),
        ] {
            writer
                .write_point(las::Point {
                    x,
                    y: 5.0,
                    z,
                    classification,
                    ..Default::default()
                })
                .unwrap();
        }
        writer.close().unwrap();
        let dtm = Raster::filled(
            GridSpec {
                west: 0.0,
                north: 10.0,
                pixel_size: 1.0,
                width: 10,
                height: 10,
            },
            30.0f32,
        );
        let points = read_points(&[path], &dtm).unwrap();
        assert_eq!(points.len(), 2, "{points:?}");
        assert!(points[0].ground && points[0].height_m == 0.0);
        assert!(!points[1].ground && (points[1].height_m - 1.5).abs() < 1e-6);
    }

    #[test]
    fn a_cell_with_few_returns_says_nothing() {
        assert_eq!(class_of(&[0.0, 0.0, 12.0]), None);
    }

    #[test]
    fn majority_removes_a_single_cell() {
        let g = grid(5);
        let mut classes = Raster::filled(g, Some(VegetationClass::Forest));
        classes.set(2, 2, Some(VegetationClass::Impassable));
        assert!(
            majority(&classes)
                .data
                .iter()
                .all(|c| *c == Some(VegetationClass::Forest))
        );
    }

    #[test]
    fn a_thicket_becomes_one_area_with_its_hole() {
        // 1:4000: the smallest area is 36 m², four cells.
        let g = grid(20);
        let mut classes = Raster::filled(g, Some(VegetationClass::Forest));
        for y in 4..16 {
            for x in 4..16 {
                classes.set(x, y, Some(VegetationClass::Impassable));
            }
        }
        for y in 8..12 {
            for x in 8..12 {
                classes.set(x, y, Some(VegetationClass::Open));
            }
        }
        let areas = vegetation_areas(&classes, 4000);
        let thicket = &areas
            .iter()
            .find(|(c, _)| *c == VegetationClass::Impassable)
            .unwrap()
            .1;
        assert_eq!(thicket.0.len(), 1, "{thicket:?}");
        assert_eq!(thicket.0[0].interiors().len(), 1, "{thicket:?}");
        // 36 m × 36 m minus the 12 m × 12 m clearing, give or take the cut corners.
        let area = thicket.unsigned_area();
        assert!((area - (1296.0 - 144.0)).abs() < 80.0, "{area}");
        let open = &areas
            .iter()
            .find(|(c, _)| *c == VegetationClass::Open)
            .unwrap()
            .1;
        assert_eq!(open.0.len(), 1);
    }

    #[test]
    fn patches_below_the_minimum_area_are_left_out() {
        let g = grid(10);
        let mut classes = Raster::filled(g, Some(VegetationClass::Forest));
        // Two cells, 18 m²: after the majority filter nothing is left anyway, and on a
        // schoolyard map (1:1000, 2.25 m²) a block of four stays.
        classes.set(5, 5, Some(VegetationClass::Walk));
        classes.set(5, 6, Some(VegetationClass::Walk));
        assert!(
            vegetation_areas(&classes, 4000)
                .iter()
                .all(|(c, _)| *c != VegetationClass::Walk)
        );
        for (x, y) in [(5, 5), (6, 5), (5, 6), (6, 6)] {
            classes.set(x, y, Some(VegetationClass::Walk));
        }
        assert!(
            vegetation_areas(&classes, 1000)
                .iter()
                .any(|(c, _)| *c == VegetationClass::Walk)
        );
    }
}
