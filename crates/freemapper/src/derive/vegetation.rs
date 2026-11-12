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
use crate::crs::BoundingBox;
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
    /// The only echo of its pulse: a roof or a wall gives one, a crown several.
    pub single: bool,
}

/// Returns higher than this decide whether a cell is a structure.
const STRUCTURE_ABOVE_M: f64 = 2.5;
/// From this share of single echoes among them, a cell is a roof or a wall, not a crown: in
/// Sanssouci (LGB 2021) 70 to 100 % on roofs, up to 30 % in crowns.
const STRUCTURE_SINGLE_SHARE: f64 = 0.6;

/// Points read per chunk.
const CHUNK_POINTS: u64 = 1_000_000;

/// Returns this close to a building are left out: walls, eaves and roof edges, which LGB does not
/// tell from vegetation (its classes are only ground, first echoes and last echoes).
const BUILDING_MARGIN_M: f64 = 1.0;
/// The cell of [`BuildingMask`].
const MASK_CELL_M: f64 = 0.5;

/// Where the returns are left out: on the buildings and within [`BUILDING_MARGIN_M`] of them.
pub struct BuildingMask {
    grid: GridSpec,
    data: Vec<bool>,
}

impl BuildingMask {
    /// The mask of `footprints` over `bbox`, both in the map's CRS.
    pub fn new(footprints: &[geo_types::Polygon<f64>], bbox: &BoundingBox) -> Self {
        use geo::{BoundingRect as _, Distance as _, Euclidean};
        let grid = GridSpec::covering(bbox, MASK_CELL_M);
        let mut data = vec![false; grid.width as usize * grid.height as usize];
        for footprint in footprints {
            let Some(rect) = footprint.bounding_rect() else {
                continue;
            };
            let col = |e: f64| ((e - grid.west) / grid.pixel_size).floor();
            let row = |n: f64| ((grid.north - n) / grid.pixel_size).floor();
            let clamp = |v: f64, max: u32| v.clamp(0.0, f64::from(max) - 1.0) as u32;
            let (x0, x1) = (
                clamp(col(rect.min().x - BUILDING_MARGIN_M), grid.width),
                clamp(col(rect.max().x + BUILDING_MARGIN_M), grid.width),
            );
            let (y0, y1) = (
                clamp(row(rect.max().y + BUILDING_MARGIN_M), grid.height),
                clamp(row(rect.min().y - BUILDING_MARGIN_M), grid.height),
            );
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let cell = &mut data[y as usize * grid.width as usize + x as usize];
                    if *cell {
                        continue;
                    }
                    let centre = geo_types::Point::new(
                        grid.west + (f64::from(x) + 0.5) * grid.pixel_size,
                        grid.north - (f64::from(y) + 0.5) * grid.pixel_size,
                    );
                    *cell = Euclidean.distance(&centre, footprint) <= BUILDING_MARGIN_M;
                }
            }
        }
        Self { grid, data }
    }

    /// Whether a return at `(e, n)` is left out.
    pub fn contains(&self, e: f64, n: f64) -> bool {
        let x = ((e - self.grid.west) / self.grid.pixel_size).floor();
        let y = ((self.grid.north - n) / self.grid.pixel_size).floor();
        x >= 0.0
            && y >= 0.0
            && x < f64::from(self.grid.width)
            && y < f64::from(self.grid.height)
            && self.data[y as usize * self.grid.width as usize + x as usize]
    }
}

/// Hands every return of a classified LAS/LAZ tile over `dtm` to `each`, with its height above
/// it, chunk by chunk. Buildings, noise and water are left out, and so is every return over a
/// cell of the DTM without data.
pub fn read_tile(
    path: &std::path::Path,
    dtm: &Raster<f32>,
    each: &mut dyn FnMut(VegetationPoint),
) -> Result<(), DeriveError> {
    use las::point::Classification;
    let grid = dtm.grid;
    let las_error = |e: las::Error| DeriveError::Image(format!("{}: {e}", path.display()));
    let mut reader = las::Reader::from_path(path).map_err(las_error)?;
    // A tile beside the area is not read at all.
    let bounds = reader.header().bounds();
    if bounds.max.x < grid.west
        || bounds.min.x > grid.west + f64::from(grid.width) * grid.pixel_size
        || bounds.max.y < grid.north - f64::from(grid.height) * grid.pixel_size
        || bounds.min.y > grid.north
    {
        return Ok(());
    }
    let mut points = las::PointDataBuilder::new()
        .for_header(reader.header())
        .build();
    while reader
        .fill_points(CHUNK_POINTS, &mut points)
        .map_err(las_error)?
        > 0
    {
        for point in points.points() {
            let point = point.map_err(las_error)?;
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
            each(VegetationPoint {
                e: point.x,
                n: point.y,
                height_m: point.z - f64::from(ground),
                ground: point.classification == Classification::Ground,
                single: point.number_of_returns <= 1,
            });
        }
    }
    Ok(())
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

    /// The class of an ISSprOM symbol of [`Self::code`].
    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.code() == code)
    }

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
    /// Returns above [`STRUCTURE_ABOVE_M`], and those of them that were single echoes.
    high: usize,
    high_single: usize,
    ground: f64,
    undergrowth: f64,
    vegetation: f64,
}

/// The returns per cell of a grid, counted as the tiles are read: a dense tile is never held in
/// memory, only these counts.
pub struct CellCounts {
    grid: GridSpec,
    counts: Vec<Counts>,
}

impl CellCounts {
    pub fn new(grid: GridSpec) -> Self {
        Self {
            grid,
            counts: vec![Counts::default(); grid.width as usize * grid.height as usize],
        }
    }

    /// Counts `p` in its cell; a return outside the grid is left out.
    pub fn add(&mut self, p: VegetationPoint) {
        let grid = self.grid;
        let x = ((p.e - grid.west) / grid.pixel_size).floor();
        let y = ((grid.north - p.n) / grid.pixel_size).floor();
        if x < 0.0 || y < 0.0 || x >= f64::from(grid.width) || y >= f64::from(grid.height) {
            return;
        }
        let c = &mut self.counts[y as usize * grid.width as usize + x as usize];
        c.all += 1;
        if p.height_m < OPEN_BELOW_M || p.ground {
            c.low += 1;
        }
        if !p.ground && p.height_m > STRUCTURE_ABOVE_M {
            c.high += 1;
            c.high_single += usize::from(p.single);
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

    /// Counts the returns of a tile ([`read_tile`]) outside `mask`.
    pub fn add_tile(
        &mut self,
        path: &std::path::Path,
        dtm: &Raster<f32>,
        mask: Option<&BuildingMask>,
    ) -> Result<(), DeriveError> {
        read_tile(path, dtm, &mut |p| {
            if !mask.is_some_and(|m| m.contains(p.e, p.n)) {
                self.add(p);
            }
        })
    }

    /// The class of every cell; `None` where too few returns fell.
    pub fn classes(&self) -> Raster<Option<VegetationClass>> {
        Raster {
            grid: self.grid,
            data: self.counts.iter().map(cell_class).collect(),
        }
    }
}

/// The class of every cell of `grid`; `None` where too few returns fall.
pub fn classify(
    points: impl IntoIterator<Item = VegetationPoint>,
    grid: GridSpec,
) -> Raster<Option<VegetationClass>> {
    let mut counts = CellCounts::new(grid);
    for p in points {
        counts.add(p);
    }
    counts.classes()
}

fn cell_class(c: &Counts) -> Option<VegetationClass> {
    if c.all < MIN_RETURNS {
        return None;
    }
    // A roof, also one the buildings of the cadastre no longer have (the scan can be years
    // older): no vegetation.
    if c.high >= MIN_RETURNS && c.high_single as f64 >= STRUCTURE_SINGLE_SHARE * c.high as f64 {
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

/// The smallest area in ground m² at 1:`scale_denominator`: 1.5 mm × 1.5 mm on paper.
pub fn min_area_m2(scale_denominator: u32) -> f64 {
    let metres_per_mm = f64::from(scale_denominator) / 1000.0;
    MIN_AREA_MM2 * metres_per_mm * metres_per_mm
}

/// The areas of every class, after the majority filter, simplified, without patches and holes
/// smaller than 1.5 mm × 1.5 mm on paper.
pub fn vegetation_areas(
    classes: &Raster<Option<VegetationClass>>,
    scale_denominator: u32,
) -> Vec<(VegetationClass, geo_types::MultiPolygon<f64>)> {
    let smoothed = majority(classes);
    let min_area_m2 = min_area_m2(scale_denominator);
    VegetationClass::ALL
        .into_iter()
        .filter_map(|class| {
            let areas = outlines(&smoothed, class, min_area_m2);
            (!areas.0.is_empty()).then_some((class, areas))
        })
        .collect()
}

/// The areas as a GeoJSON FeatureCollection in the map's CRS, one feature per class with its
/// ISSprOM `code`.
pub fn vegetation_to_geojson(areas: &[(VegetationClass, geo_types::MultiPolygon<f64>)]) -> String {
    let ring = |r: &geo_types::LineString<f64>| r.0.iter().map(|c| [c.x, c.y]).collect::<Vec<_>>();
    let features: Vec<serde_json::Value> = areas
        .iter()
        .map(|(class, area)| {
            let polygons: Vec<Vec<Vec<[f64; 2]>>> = area
                .0
                .iter()
                .map(|p| {
                    std::iter::once(ring(p.exterior()))
                        .chain(p.interiors().iter().map(ring))
                        .collect()
                })
                .collect();
            serde_json::json!({
                "type": "Feature",
                "properties": { "code": class.code() },
                "geometry": { "type": "MultiPolygon", "coordinates": polygons }
            })
        })
        .collect();
    serde_json::json!({ "type": "FeatureCollection", "features": features }).to_string()
}

/// Reads the areas written by [`vegetation_to_geojson`].
pub fn vegetation_from_geojson(
    text: &str,
) -> Result<Vec<(VegetationClass, geo_types::MultiPolygon<f64>)>, String> {
    let collection: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let features = collection["features"].as_array().ok_or("no features")?;
    let ring = |r: &serde_json::Value| -> Result<geo_types::LineString<f64>, String> {
        r.as_array()
            .ok_or("a ring is no array")?
            .iter()
            .map(|c| match (c[0].as_f64(), c[1].as_f64()) {
                (Some(e), Some(n)) => Ok((e, n)),
                _ => Err("a coordinate is no pair of numbers".to_string()),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(geo_types::LineString::from)
    };
    features
        .iter()
        .map(|f| {
            let code = f["properties"]["code"].as_str().unwrap_or_default();
            let class = VegetationClass::from_code(code)
                .ok_or_else(|| format!("unknown vegetation code `{code}`"))?;
            let polygons = f["geometry"]["coordinates"]
                .as_array()
                .ok_or("no MultiPolygon")?
                .iter()
                .map(|p| {
                    let rings = p.as_array().ok_or("a polygon is no array")?;
                    let (outer, holes) = rings.split_first().ok_or("a polygon without rings")?;
                    Ok(geo_types::Polygon::new(
                        ring(outer)?,
                        holes.iter().map(ring).collect::<Result<_, String>>()?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok((class, geo_types::MultiPolygon(polygons)))
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
                single: false,
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
    fn read_tile_measures_above_the_dtm_and_skips_buildings() {
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
        let mut points = Vec::new();
        read_tile(&path, &dtm, &mut |p| points.push(p)).unwrap();
        assert_eq!(points.len(), 2, "{points:?}");
        assert!(points[0].ground && points[0].height_m == 0.0);
        assert!(!points[1].ground && (points[1].height_m - 1.5).abs() < 1e-6);
    }

    #[test]
    fn counting_a_tile_gives_the_classes_of_its_points() {
        use las::point::Classification;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tile.laz");
        let header = las::Builder::from((1, 2)).into_header().unwrap();
        let mut writer = las::Writer::from_path(&path, header).unwrap();
        // Cell (0, 0): a lawn; cell (1, 0): dense bushes.
        for i in 0..20 {
            let x = 0.1 + 0.1 * f64::from(i);
            for (e, z, classification) in [
                (x, 30.0, Classification::Ground),
                (CELL_M + x, 31.2, Classification::MediumVegetation),
            ] {
                writer
                    .write_point(las::Point {
                        x: e,
                        y: 1.5,
                        z,
                        classification,
                        ..Default::default()
                    })
                    .unwrap();
            }
        }
        writer.close().unwrap();
        let g = GridSpec {
            west: 0.0,
            north: CELL_M,
            pixel_size: CELL_M,
            width: 2,
            height: 1,
        };
        let dtm = Raster::filled(g, 30.0f32);
        let mut counts = CellCounts::new(g);
        counts.add_tile(&path, &dtm, None).unwrap();
        let classes = counts.classes();
        assert_eq!(classes.get(0, 0), Some(VegetationClass::Open));
        assert_eq!(classes.get(1, 0), Some(VegetationClass::Impassable));
        let mut points = Vec::new();
        read_tile(&path, &dtm, &mut |p| points.push(p)).unwrap();
        assert_eq!(classify(points, g).data, classes.data);
        // A tile beside the area gives nothing (and is not read).
        let beside = Raster::filled(GridSpec { west: 1000.0, ..g }, 30.0f32);
        let mut none = 0;
        read_tile(&path, &beside, &mut |_| none += 1).unwrap();
        assert_eq!(none, 0);
        // A building just east of the bushes takes them out: within a metre of it, a return is a
        // wall or an eave.
        let building = geo_types::Polygon::new(
            geo_types::LineString::from(vec![
                (2.0 * CELL_M + 0.5, 0.0),
                (2.0 * CELL_M + 5.0, 0.0),
                (2.0 * CELL_M + 5.0, CELL_M),
                (2.0 * CELL_M + 0.5, CELL_M),
                (2.0 * CELL_M + 0.5, 0.0),
            ]),
            vec![],
        );
        let bbox = BoundingBox {
            min_e: 0.0,
            min_n: 0.0,
            max_e: 2.0 * CELL_M,
            max_n: CELL_M,
        };
        let mask = BuildingMask::new(&[building], &bbox);
        assert!(mask.contains(2.0 * CELL_M - 0.2, 1.5) && !mask.contains(CELL_M + 0.5, 1.5));
        let mut counts = CellCounts::new(g);
        counts.add_tile(&path, &dtm, Some(&mask)).unwrap();
        let classes = counts.classes();
        assert_eq!(classes.get(0, 0), Some(VegetationClass::Open));
        assert_eq!(
            classes.get(1, 0),
            Some(VegetationClass::Impassable),
            "the west part of the cell is still counted"
        );
    }

    #[test]
    fn the_geojson_has_one_feature_per_class_with_its_holes() {
        let square = |w: f64, s: f64, side: f64| {
            geo_types::LineString::from(vec![
                (w, s),
                (w + side, s),
                (w + side, s + side),
                (w, s + side),
                (w, s),
            ])
        };
        let thicket = geo_types::MultiPolygon(vec![geo_types::Polygon::new(
            square(0.0, 0.0, 30.0),
            vec![square(10.0, 10.0, 6.0)],
        )]);
        let lawn = geo_types::MultiPolygon(vec![geo_types::Polygon::new(
            square(40.0, 0.0, 9.0),
            vec![],
        )]);
        let text = vegetation_to_geojson(&[
            (VegetationClass::Impassable, thicket),
            (VegetationClass::Open, lawn),
        ]);
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        let features = json["features"].as_array().unwrap();
        assert_eq!(features.len(), 2);
        assert_eq!(features[0]["properties"]["code"], "410");
        assert_eq!(features[1]["properties"]["code"], "401");
        let polygon = &features[0]["geometry"]["coordinates"][0];
        assert_eq!(features[0]["geometry"]["type"], "MultiPolygon");
        assert_eq!(polygon.as_array().unwrap().len(), 2, "outline and hole");
        assert_eq!(polygon[1][0], serde_json::json!([10.0, 10.0]));
        let back = vegetation_from_geojson(&text).unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].0, VegetationClass::Impassable);
        assert_eq!(back[0].1.0[0].interiors().len(), 1);
        assert!(
            vegetation_from_geojson(r#"{"features":[{"properties":{"code":"999"}}]}"#).is_err()
        );
    }

    #[test]
    fn a_roof_is_no_vegetation_but_a_crown_is() {
        let g = grid(2);
        let mut points = Vec::new();
        for (x, single) in [(0, true), (1, false)] {
            let mut cell = cell_points(x, 0, &[7.0; 12], &g);
            cell.extend(cell_points(x, 0, &[0.0; 4], &g));
            for p in &mut cell {
                p.single = single && !p.ground;
            }
            points.extend(cell);
        }
        let classes = classify(points, g);
        assert_eq!(classes.get(0, 0), None, "a roof");
        assert_eq!(classes.get(1, 0), Some(VegetationClass::Forest), "a crown");
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
