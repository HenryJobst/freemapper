//! Georeferenced rasters in EPSG:25833 with world-file georeferencing.

use crate::crs::BoundingBox;

/// Tolerance in pixels for snapping and grid alignment.
const PIXEL_EPSILON: f64 = 1e-6;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RasterError {
    #[error("invalid world file: {0}")]
    WorldFile(String),
    #[error("rasters are not aligned to the same pixel grid: {0}")]
    Misaligned(String),
    #[error("rasters have different pixel sizes: {0}")]
    PixelSizeMismatch(String),
}

/// Axis-parallel pixel grid: north-west corner, square pixel size in metres, size in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridSpec {
    pub west: f64,
    pub north: f64,
    pub pixel_size: f64,
    pub width: u32,
    pub height: u32,
}

impl GridSpec {
    /// Reads a world file (`A D B E C F`, one per line). C/F give the centre of the top-left pixel.
    pub fn from_world_file(text: &str, width: u32, height: u32) -> Result<Self, RasterError> {
        let values: Vec<f64> = text
            .split_whitespace()
            .map(|v| {
                v.parse()
                    .map_err(|_| RasterError::WorldFile(format!("not a number: {v:?}")))
            })
            .collect::<Result<_, _>>()?;
        let [a, d, b, e, c, f] = values[..] else {
            return Err(RasterError::WorldFile(format!(
                "expected 6 values, found {}",
                values.len()
            )));
        };
        if d != 0.0 || b != 0.0 {
            return Err(RasterError::WorldFile(
                "rotated rasters are not supported".into(),
            ));
        }
        if a <= 0.0 || (a + e).abs() > a * PIXEL_EPSILON {
            return Err(RasterError::WorldFile(format!(
                "pixels must be square and north-up (A {a}, E {e})"
            )));
        }
        Ok(Self {
            west: c - a / 2.0,
            north: f + a / 2.0,
            pixel_size: a,
            width,
            height,
        })
    }

    /// World file text for this grid, 10 decimals like the LGB files.
    pub fn world_file(&self) -> String {
        let p = self.pixel_size;
        format!(
            "{p:.10}\n0.0000000000\n0.0000000000\n{:.10}\n{:.10}\n{:.10}\n",
            -p,
            self.west + p / 2.0,
            self.north - p / 2.0
        )
    }

    pub fn bounds(&self) -> BoundingBox {
        BoundingBox {
            min_e: self.west,
            min_n: self.north - f64::from(self.height) * self.pixel_size,
            max_e: self.west + f64::from(self.width) * self.pixel_size,
            max_n: self.north,
        }
    }

    /// Smallest grid of `pixel_size` with edges on multiples of `pixel_size` that covers `bbox`.
    pub fn covering(bbox: &BoundingBox, pixel_size: f64) -> Self {
        let lower = |v: f64| (v / pixel_size + PIXEL_EPSILON).floor();
        let upper = |v: f64| (v / pixel_size - PIXEL_EPSILON).ceil();
        let (west, east) = (lower(bbox.min_e), upper(bbox.max_e));
        let (south, north) = (lower(bbox.min_n), upper(bbox.max_n));
        Self {
            west: west * pixel_size,
            north: north * pixel_size,
            pixel_size,
            width: (east - west) as u32,
            height: (north - south) as u32,
        }
    }

    /// Offset of `other`'s north-west pixel in this grid, in whole pixels.
    ///
    /// Fails if the grids differ in pixel size or are not aligned to within 1 % of a pixel.
    fn offset_of(&self, other: &GridSpec) -> Result<(i64, i64), RasterError> {
        if (self.pixel_size - other.pixel_size).abs() >= self.pixel_size * PIXEL_EPSILON {
            return Err(RasterError::PixelSizeMismatch(format!(
                "{} m vs {} m",
                self.pixel_size, other.pixel_size
            )));
        }
        let to_pixels = |metres: f64| {
            let pixels = metres / self.pixel_size;
            if (pixels - pixels.round()).abs() < 0.01 {
                Ok(pixels.round() as i64)
            } else {
                Err(RasterError::Misaligned(format!("{self:?} vs {other:?}")))
            }
        };
        Ok((
            to_pixels(other.west - self.west)?,
            to_pixels(self.north - other.north)?,
        ))
    }
}

/// Pixels of a grid, row by row from the north-west corner.
#[derive(Debug, Clone, PartialEq)]
pub struct Raster<T> {
    pub grid: GridSpec,
    pub data: Vec<T>,
}

impl<T: Copy> Raster<T> {
    pub fn filled(grid: GridSpec, value: T) -> Self {
        Self {
            grid,
            data: vec![value; grid.width as usize * grid.height as usize],
        }
    }

    pub fn get(&self, x: u32, y: u32) -> T {
        self.data[self.index(x, y)]
    }

    pub fn set(&mut self, x: u32, y: u32, value: T) {
        let i = self.index(x, y);
        self.data[i] = value;
    }

    fn index(&self, x: u32, y: u32) -> usize {
        assert!(
            x < self.grid.width && y < self.grid.height,
            "({x}, {y}) outside {:?}",
            self.grid
        );
        y as usize * self.grid.width as usize + x as usize
    }

    /// Copies the pixels of `other` where both rasters overlap.
    pub fn paste(&mut self, other: &Raster<T>) -> Result<(), RasterError> {
        self.paste_with(&other.grid, |x, y| other.get(x, y))
    }

    /// Like [`Self::paste`] for a source given as grid plus pixel accessor.
    pub fn paste_with(
        &mut self,
        source: &GridSpec,
        pixel: impl Fn(u32, u32) -> T,
    ) -> Result<(), RasterError> {
        let (dx, dy) = self.grid.offset_of(source)?;
        let x_range = dx.max(0)..(dx + i64::from(source.width)).min(i64::from(self.grid.width));
        let y_range = dy.max(0)..(dy + i64::from(source.height)).min(i64::from(self.grid.height));
        for y in y_range {
            for x in x_range.clone() {
                let value = pixel((x - dx) as u32, (y - dy) as u32);
                self.set(x as u32, y as u32, value);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOP_WORLD_FILE: &str = "0.2000000000\n0.0000000000\n0.0000000000\n-0.2000000000\n421000.1000000000\n5821999.9000000004\n";

    fn grid(west: f64, north: f64, width: u32, height: u32) -> GridSpec {
        GridSpec {
            west,
            north,
            pixel_size: 1.0,
            width,
            height,
        }
    }

    #[test]
    fn world_file_roundtrip_lgb() {
        let g = GridSpec::from_world_file(DOP_WORLD_FILE, 5000, 5000).unwrap();
        assert!((g.west - 421_000.0).abs() < 1e-6, "{g:?}");
        assert!((g.north - 5_822_000.0).abs() < 1e-6, "{g:?}");
        assert!((g.pixel_size - 0.2).abs() < 1e-12);
        assert!((g.bounds().max_e - 422_000.0).abs() < 1e-6);
        assert!((g.bounds().min_n - 5_821_000.0).abs() < 1e-6);
        let again = GridSpec::from_world_file(&g.world_file(), 5000, 5000).unwrap();
        assert!((again.west - g.west).abs() < 1e-6 && (again.north - g.north).abs() < 1e-6);
    }

    #[test]
    fn rotated_world_file_is_rejected() {
        let rotated = "0.2\n0.01\n0\n-0.2\n421000.1\n5821999.9\n";
        assert!(GridSpec::from_world_file(rotated, 10, 10).is_err());
        let not_square = "0.2\n0\n0\n-0.4\n421000.1\n5821999.9\n";
        assert!(GridSpec::from_world_file(not_square, 10, 10).is_err());
    }

    #[test]
    fn covering_snaps_to_pixel_grid() {
        let bbox = BoundingBox {
            min_e: 421_100.05,
            min_n: 5_821_100.05,
            max_e: 421_101.0,
            max_n: 5_821_101.0,
        };
        let g = GridSpec::covering(&bbox, 0.2);
        assert!((g.west - 421_100.0).abs() < 1e-6, "{g:?}");
        assert!((g.north - 5_821_101.0).abs() < 1e-6, "{g:?}");
        assert_eq!((g.width, g.height), (5, 5));
    }

    #[test]
    fn paste_two_tiles_side_by_side_is_seamless() {
        let west = Raster::filled(grid(0.0, 10.0, 10, 10), 1u8);
        let east = Raster::filled(grid(10.0, 10.0, 10, 10), 2u8);
        let mut target = Raster::filled(grid(0.0, 10.0, 20, 10), 0u8);
        target.paste(&west).unwrap();
        target.paste(&east).unwrap();
        for y in 0..10 {
            for x in 0..20 {
                assert_eq!(target.get(x, y), if x < 10 { 1 } else { 2 }, "({x}, {y})");
            }
        }
    }

    #[test]
    fn misaligned_tiles_are_an_error() {
        let mut target = Raster::filled(grid(0.0, 10.0, 10, 10), 0u8);
        let shifted = Raster::filled(grid(0.5, 10.0, 10, 10), 1u8);
        assert!(matches!(
            target.paste(&shifted),
            Err(RasterError::Misaligned(_))
        ));
        let mut coarse_grid = grid(0.0, 10.0, 5, 5);
        coarse_grid.pixel_size = 2.0;
        let coarse = Raster::filled(coarse_grid, 1u8);
        assert!(matches!(
            target.paste(&coarse),
            Err(RasterError::PixelSizeMismatch(_))
        ));
        assert!(target.data.iter().all(|v| *v == 0), "nothing pasted");
    }

    #[test]
    fn paste_copies_only_the_overlap() {
        let source = Raster::filled(grid(-5.0, 15.0, 10, 10), 7u8);
        let mut target = Raster::filled(grid(0.0, 10.0, 10, 10), 0u8);
        target.paste(&source).unwrap();
        assert_eq!(target.get(0, 0), 7);
        assert_eq!(target.get(4, 4), 7);
        assert_eq!(target.get(5, 5), 0);
    }
}
