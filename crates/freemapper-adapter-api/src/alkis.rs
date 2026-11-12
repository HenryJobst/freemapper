//! ALKIS buildings and land use: the area they are fetched for, what a source delivers, and
//! the trait a source implements.

use freemapper_georef::crs::{BoundingBox, Crs, ProjectedPoint};
use serde_json::Value;

use crate::env::AdapterEnv;
use crate::error::AdapterError;

/// The area ALKIS is fetched for: its EPSG:25833 box, which decides the regions, and the map's
/// CRS with the box in it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FetchArea {
    pub region_bbox: BoundingBox,
    pub crs: Crs,
    /// The area in `crs`.
    pub bbox: BoundingBox,
}

impl FetchArea {
    pub fn new(region_bbox: BoundingBox, crs: Crs) -> Self {
        Self {
            region_bbox,
            crs,
            bbox: bbox_in(&region_bbox, Crs::Utm33, crs),
        }
    }

    /// The area in `crs`: exact for the map's CRS and EPSG:25833, else the envelope of the
    /// region box's outline in `crs`.
    pub fn bbox_for(&self, crs: Crs) -> BoundingBox {
        if crs == self.crs {
            self.bbox
        } else {
            bbox_in(&self.region_bbox, Crs::Utm33, crs)
        }
    }
}

/// The envelope of `bbox` (in `from`) in `to`, from points along its outline, grown by 1 m.
fn bbox_in(bbox: &BoundingBox, from: Crs, to: Crs) -> BoundingBox {
    if from == to {
        return *bbox;
    }
    const STEPS: u32 = 8;
    let mut out: Option<BoundingBox> = None;
    for i in 0..=STEPS {
        let t = f64::from(i) / f64::from(STEPS);
        let e = bbox.min_e + t * (bbox.max_e - bbox.min_e);
        let n = bbox.min_n + t * (bbox.max_n - bbox.min_n);
        for (pe, pn) in [
            (e, bbox.min_n),
            (e, bbox.max_n),
            (bbox.min_e, n),
            (bbox.max_e, n),
        ] {
            let Some(p) = to.reproject(
                from,
                ProjectedPoint {
                    easting: pe,
                    northing: pn,
                },
            ) else {
                continue;
            };
            let b = out.get_or_insert(BoundingBox {
                min_e: p.easting,
                min_n: p.northing,
                max_e: p.easting,
                max_n: p.northing,
            });
            b.min_e = b.min_e.min(p.easting);
            b.min_n = b.min_n.min(p.northing);
            b.max_e = b.max_e.max(p.easting);
            b.max_n = b.max_n.max(p.northing);
        }
    }
    out.map_or(*bbox, |b| b.buffered(1.0))
}

/// The ALKIS objects of one region in an area: GeoJSON features in the LGB schema (properties
/// `oid`, `aktualit`, `gebnutzbez`, `funktion`, `nutzart`, `bez` in the LGB spellings), with
/// the CRS of their coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct AlkisPart {
    pub features: Vec<Value>,
    pub crs: Crs,
}

/// The ALKIS service of a region. The engine merges the parts of all regions an area touches
/// (an object two regions deliver is kept once) and moves them into the CRS of the map.
pub trait AlkisSource: Send + Sync {
    /// The buildings and structures in `area`.
    fn buildings(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError>;

    /// The land use in `area`.
    fn land_use(
        &self,
        env: &mut AdapterEnv<'_>,
        area: &FetchArea,
    ) -> Result<AlkisPart, AdapterError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_area_in_another_crs_is_the_envelope_of_its_outline() {
        // Hamburg city hall, 1 km square in EPSG:25833 (zone 33 coordinates of a zone 32 place).
        let region_bbox = BoundingBox {
            min_e: 165_000.0,
            min_n: 5_945_000.0,
            max_e: 166_000.0,
            max_n: 5_946_000.0,
        };
        let area = FetchArea::new(region_bbox, Crs::Utm32);
        // The map's CRS and EPSG:25833 are exact.
        assert_eq!(area.bbox_for(Crs::Utm33), region_bbox);
        assert_eq!(area.bbox_for(Crs::Utm32), area.bbox);
        // The two grids are turned against each other, so the envelope is larger than the
        // square: more than 1 km a side, less than 1.2 km.
        let (width, height) = (area.bbox.width(), area.bbox.height());
        assert!(width > 1_000.0 && width < 1_200.0, "{width}");
        assert!(height > 1_000.0 && height < 1_200.0, "{height}");
        // A map in EPSG:25833 keeps its box.
        assert_eq!(FetchArea::new(region_bbox, Crs::Utm33).bbox, region_bbox);
    }
}
