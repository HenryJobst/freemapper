// SPDX-License-Identifier: AGPL-3.0-or-later

//! ESRI ASCII grids (`.asc`), the format of the Polish height models (GUGiK NMT and NMPT). The
//! grid is read as it lies, in the grid of its source (PL-1992 for GUGiK); [`crate::warp`] moves
//! it into the grid of a project.

use crate::raster::{GridSpec, Raster};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum AscError {
    #[error("the header has no {0}")]
    MissingKey(&'static str),
    #[error("the header value of {key} is not a number: {value:?}")]
    BadHeader { key: &'static str, value: String },
    #[error("the pixels must be square and positive (cellsize {0})")]
    CellSize(f64),
    #[error("the grid holds {found} values, the header says {expected}")]
    Count { expected: usize, found: usize },
    #[error("not a number in the grid: {0:?}")]
    BadValue(String),
}

/// The keys of the header. Anything else is where the values begin, so a grid that starts with
/// `nan` is not taken for a key.
const KEYS: [&str; 8] = [
    "ncols",
    "nrows",
    "xllcorner",
    "xllcenter",
    "yllcorner",
    "yllcenter",
    "cellsize",
    "nodata_value",
];

/// Reads an ESRI ASCII grid. Nodata becomes `NaN`. `xllcenter`/`yllcenter` and
/// `xllcorner`/`yllcorner` both work; the grid starts at the north-west corner like [`Raster`].
/// Without `NODATA_value` every value is a height, also -9999 (the ESRI default is not assumed).
pub fn read_asc(bytes: &[u8]) -> Result<Raster<f32>, AscError> {
    let text = String::from_utf8_lossy(bytes);
    let mut tokens = text.split_ascii_whitespace().peekable();
    let mut header = std::collections::HashMap::new();
    while let Some(key) = tokens.next_if(|t| KEYS.contains(&t.to_ascii_lowercase().as_str())) {
        let value = tokens.next().unwrap_or_default();
        header.insert(key.to_ascii_lowercase(), value);
    }
    let number = |key: &'static str| -> Result<Option<f64>, AscError> {
        header
            .get(key)
            .map(|v| {
                v.parse().map_err(|_| AscError::BadHeader {
                    key,
                    value: (*v).to_string(),
                })
            })
            .transpose()
    };
    let required = |key: &'static str| number(key)?.ok_or(AscError::MissingKey(key));
    // A count is a whole number above zero; anything else would be cut to one by `as`.
    let count = |key: &'static str| -> Result<u32, AscError> {
        let v = required(key)?;
        if v >= 1.0 && v.fract() == 0.0 && v <= f64::from(u32::MAX) {
            Ok(v as u32)
        } else {
            Err(AscError::BadHeader {
                key,
                value: v.to_string(),
            })
        }
    };
    let cols = count("ncols")?;
    let rows = count("nrows")?;
    let cell = required("cellsize")?;
    if cell <= 0.0 || !cell.is_finite() {
        return Err(AscError::CellSize(cell));
    }
    // The corner of a cell is half a cell from its centre.
    let west = match number("xllcorner")? {
        Some(x) => x,
        None => required("xllcenter")? - cell / 2.0,
    };
    let south = match number("yllcorner")? {
        Some(y) => y,
        None => required("yllcenter")? - cell / 2.0,
    };
    let nodata = number("nodata_value")?.map(|v| v as f32);

    let expected =
        usize::try_from(u64::from(cols) * u64::from(rows)).map_err(|_| AscError::BadHeader {
            key: "nrows",
            value: rows.to_string(),
        })?;
    // A damaged header must not ask for gigabytes before the first value is read.
    let mut data = Vec::with_capacity(expected.min(1 << 24));
    for token in tokens {
        let z: f32 = token
            .parse()
            .map_err(|_| AscError::BadValue(token.to_string()))?;
        data.push(if Some(z) == nodata { f32::NAN } else { z });
    }
    if data.len() != expected {
        return Err(AscError::Count {
            expected,
            found: data.len(),
        });
    }
    let grid = GridSpec {
        west,
        north: south + f64::from(rows) * cell,
        pixel_size: cell,
        width: cols,
        height: rows,
    };
    Ok(Raster { grid, data })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CENTRE: &str = "ncols 3\nnrows 3\nxllcenter 198000.5\nyllcenter 507000.5\ncellsize 1.0\nNODATA_value -9999\n\
                          1.5 2.5 3.5\n4.5 -9999 6.5\n7.5 8.5 9.5\n";

    #[test]
    fn nodata_becomes_nan_and_rows_run_north_to_south() {
        let r = read_asc(CENTRE.as_bytes()).unwrap();
        assert_eq!((r.grid.width, r.grid.height), (3, 3));
        assert_eq!(r.get(0, 0), 1.5);
        assert_eq!(r.get(2, 0), 3.5);
        assert!(r.get(1, 1).is_nan());
        assert_eq!(r.get(2, 2), 9.5);
    }

    #[test]
    fn the_centre_of_the_first_pixel_is_on_the_header_value() {
        // yllcenter is the centre of the south row, so the north edge is 2.5 m above the
        // centre of the north row's neighbour: 507000.5 + 2 rows + half a pixel.
        let g = read_asc(CENTRE.as_bytes()).unwrap().grid;
        assert_eq!(g.pixel_size, 1.0);
        assert_eq!(g.west, 198000.0);
        assert_eq!(g.north, 507003.0);
    }

    #[test]
    fn a_corner_header_gives_the_same_grid() {
        let corner = CENTRE
            .replace("xllcenter 198000.5", "xllcorner 198000")
            .replace("yllcenter 507000.5", "yllcorner 507000");
        let a = read_asc(CENTRE.as_bytes()).unwrap();
        let b = read_asc(corner.as_bytes()).unwrap();
        assert_eq!(a.grid, b.grid);
    }

    #[test]
    fn keys_are_case_insensitive_and_a_missing_nodata_keeps_every_value() {
        let text = "NCOLS 2\nNROWS 1\nXLLCORNER 0\nYLLCORNER 0\nCELLSIZE 0.5\n-9999 3\n";
        let r = read_asc(text.as_bytes()).unwrap();
        assert_eq!(r.get(0, 0), -9999.0);
        assert_eq!(r.grid.pixel_size, 0.5);
    }

    #[test]
    fn a_short_grid_is_refused_with_both_counts() {
        let text = CENTRE.replace("8.5 9.5\n", "8.5\n");
        assert_eq!(
            read_asc(text.as_bytes()),
            Err(AscError::Count {
                expected: 9,
                found: 8
            })
        );
    }

    #[test]
    fn a_missing_header_key_and_a_bad_number_are_named() {
        let text = CENTRE.replace("cellsize 1.0\n", "");
        assert_eq!(
            read_asc(text.as_bytes()),
            Err(AscError::MissingKey("cellsize"))
        );
        let text = CENTRE.replace("ncols 3", "ncols three");
        assert!(matches!(
            read_asc(text.as_bytes()),
            Err(AscError::BadHeader { key: "ncols", .. })
        ));
        let text = CENTRE.replace("2.5", "x");
        assert_eq!(
            read_asc(text.as_bytes()),
            Err(AscError::BadValue("x".into()))
        );
    }

    #[test]
    fn a_cell_size_of_zero_is_refused() {
        let text = CENTRE.replace("cellsize 1.0", "cellsize 0");
        assert_eq!(read_asc(text.as_bytes()), Err(AscError::CellSize(0.0)));
    }

    #[test]
    fn a_grid_is_a_raster_on_the_given_grid_spec() {
        let r = read_asc(CENTRE.as_bytes()).unwrap();
        let expected = GridSpec {
            west: 198000.0,
            north: 507003.0,
            pixel_size: 1.0,
            width: 3,
            height: 3,
        };
        assert_eq!(r.grid, expected);
    }

    #[test]
    fn a_cell_larger_than_one_scales_the_extent_and_the_centre_shift() {
        let corner = "ncols 2\nnrows 3\nxllcorner 10\nyllcorner 20\ncellsize 2\n1 2 3 4 5 6\n";
        let g = read_asc(corner.as_bytes()).unwrap().grid;
        assert_eq!((g.west, g.north, g.pixel_size), (10.0, 26.0, 2.0));
        let centre = corner
            .replace("xllcorner 10", "xllcenter 11")
            .replace("yllcorner 20", "yllcenter 21");
        assert_eq!(read_asc(centre.as_bytes()).unwrap().grid, g);
    }

    #[test]
    fn a_grid_may_begin_with_a_value_that_looks_like_a_word() {
        let text = "ncols 2\nnrows 1\nxllcorner 0\nyllcorner 0\ncellsize 1\nnan 3\n";
        let r = read_asc(text.as_bytes()).unwrap();
        assert!(r.get(0, 0).is_nan());
        assert_eq!(r.get(1, 0), 3.0);
    }

    #[test]
    fn a_count_that_is_zero_negative_or_fractional_is_refused() {
        for bad in ["0", "-3", "2.5", "nan"] {
            let text = CENTRE.replace("ncols 3", &format!("ncols {bad}"));
            assert!(
                matches!(
                    read_asc(text.as_bytes()),
                    Err(AscError::BadHeader { key: "ncols", .. })
                ),
                "{bad}"
            );
        }
        let text = CENTRE.replace("nrows 3", "nrows 0");
        assert!(matches!(
            read_asc(text.as_bytes()),
            Err(AscError::BadHeader { key: "nrows", .. })
        ));
    }
}
