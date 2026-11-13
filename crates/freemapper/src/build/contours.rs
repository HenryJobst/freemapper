//! Contour candidates from the contour lines of `derive`.

use geo::{Contains as _, Distance as _, InteriorPoint as _};
use geo_types::{MultiPolygon, Polygon};

use crate::build::water::outside_water;
use crate::crs::ProjectedPoint;
use crate::derive::contours::ContourLine;
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::coord::MapCoord;
use crate::omap::{MapObject, MapObjectKind, SymbolSet, closed_ring};

/// ISSprOM contour and index contour.
pub const CONTOUR_CODE: &str = "101";
pub const INDEX_CONTOUR_CODE: &str = "102";

/// The contour lines of `derive`, with the interval they were traced at and the terrain model
/// they come from: its source and the newest currency of its tiles, as tag values.
pub struct Contours {
    pub lines: Vec<ContourLine>,
    pub interval: f64,
    pub source: String,
    pub currency: Option<String>,
}

/// 101/102 candidate objects for `contours`, within `buffer_m` of `map_area`. They stop at the
/// drawn ALKIS `water`: ISSprOM shows no contours in uncrossable water.
pub fn contour_objects(
    contours: &Contours,
    footprints: &[Polygon<f64>],
    water: &MultiPolygon<f64>,
    map_area: &Polygon<f64>,
    buffer_m: f64,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
) -> Result<Vec<MapObject>, GeorefError> {
    let Contours {
        lines,
        interval,
        source,
        currency,
    } = contours;
    let index_every = 5.0 * interval;
    let id = |code: &str| {
        symbol_set
            .id_for_code(code)
            .expect("symbol set defines contours")
    };
    let (contour, index) = (id(CONTOUR_CODE), id(INDEX_CONTOUR_CODE));
    // Stubs of a cut contour along the shore shorter than 1 mm on paper are noise.
    let min_piece_m = f64::from(georef.scale_denominator) / 1000.0;
    // DGM1 is interpolated under buildings: closed rings there are artefacts.
    let on_a_building = |ring: &geo_types::LineString<f64>| {
        geo_types::Polygon::new(ring.clone(), vec![])
            .interior_point()
            .is_some_and(|p| footprints.iter().any(|f| f.contains(&p)))
    };
    let objects = lines
        .iter()
        .filter(|line| {
            let ls = geo_types::LineString::from(line.points.clone());
            let near = geo::Euclidean.distance(&ls, map_area) <= buffer_m;
            near && !(ls.is_closed() && ls.0.len() > 3 && on_a_building(&ls))
        })
        .flat_map(|line| {
            let ls = geo_types::LineString::from(line.points.clone());
            outside_water(&ls, water, min_piece_m)
                .into_iter()
                .map(move |piece| (line, piece))
        })
        .map(|(line, piece)| {
            let coords: Vec<(i32, i32)> = piece
                .coords()
                .map(|c| {
                    georef
                        .projected_to_map(ProjectedPoint {
                            easting: c.x,
                            northing: c.y,
                        })
                        .map(|c| (c.x, c.y))
                })
                .collect::<Result<_, _>>()?;
            let closed = coords.len() > 3 && coords.first() == coords.last();
            let mut tags = vec![
                ("freemapper:source".to_string(), source.clone()),
                ("freemapper:height".to_string(), format!("{}", line.height)),
            ];
            if let Some(currency) = currency {
                tags.push(("freemapper:currency".to_string(), currency.clone()));
            }
            Ok(MapObject {
                symbol_id: if (line.height / index_every).fract() == 0.0 {
                    index
                } else {
                    contour
                },
                kind: MapObjectKind::Path,
                coords: if closed {
                    closed_ring(&coords)
                } else {
                    coords
                        .into_iter()
                        .map(|(x, y)| MapCoord::new(x, y))
                        .collect()
                },
                tags,
            })
        })
        .collect::<Result<_, GeorefError>>()?;
    Ok(objects)
}
