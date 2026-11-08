//! Regions: the states freemapper has data for, and which of them cover an area.
//!
//! Every state keeps its own ALKIS service: the LGB service returns no features inside Berlin and
//! the Berlin service none outside, so an area near a border needs several. The rasters come
//! from one provider, the one of the region containing the area's centre (LGB also serves Berlin).

use std::sync::OnceLock;

use geo::{Contains, Intersects};
use geo_types::{MultiPolygon, Point, Polygon, Rect, coord};

use crate::crs::BoundingBox;
use crate::tiles::TileId;

/// Brandenburg from LGB ALKIS `verwaltungseinheit` DE12, with Berlin as a hole (dl-de/by-2-0).
const BRANDENBURG_BOUNDARY: &str = include_str!("../../../../assets/regions/brandenburg.geojson");
/// Berlin boundary from ALKIS `alkis:land` (EPSG:25833, simplified 25 m; dl-de/zero-2-0).
const BERLIN_BOUNDARY: &str = include_str!("../../../../assets/regions/berlin.geojson");
/// Saxony from GeoSN ALKIS `VerwaltungsEinheit` DE14 (dl-de/by-2-0).
const SAXONY_BOUNDARY: &str = include_str!("../../../../assets/regions/saxony.geojson");
/// Saxony-Anhalt from LVermGeo DVG `VerwaltungsEinheit` DE15 (dl-de/by-2-0), in EPSG:25833 like
/// the others.
const SAXONY_ANHALT_BOUNDARY: &str =
    include_str!("../../../../assets/regions/saxony-anhalt.geojson");
/// Hamburg from LGV `app:landesgrenze` (dl-de/by-2-0), with Neuwerk and Scharhörn.
const HAMBURG_BOUNDARY: &str = include_str!("../../../../assets/regions/hamburg.geojson");
/// Hesse from HVBG ALKIS vereinfacht `VerwaltungsEinheit` DE06 (dl-de/zero-2-0). In EPSG:25833 it
/// has negative eastings (Hesse reaches from about -17 km to 167 km).
const HESSEN_BOUNDARY: &str = include_str!("../../../../assets/regions/hessen.geojson");
/// Schleswig-Holstein from LVermGeo SH ALKIS (simplified 25 m, EPSG:25833; dl-de/by-2-0). It shares
/// about 131 km of border with Hamburg.
const SCHLESWIG_HOLSTEIN_BOUNDARY: &str =
    include_str!("../../../../assets/regions/schleswig-holstein.geojson");
/// Mecklenburg-Vorpommern from LAiV M-V ALKIS (simplified 25 m, EPSG:25833; CC BY 4.0). It shares
/// its southern border with Brandenburg, and stays in UTM zone 33 like it.
const MECKLENBURG_VORPOMMERN_BOUNDARY: &str =
    include_str!("../../../../assets/regions/mecklenburg-vorpommern.geojson");
/// The area is widened by this much before testing, so the 25 m simplification never drops
/// buildings right at the border.
const MARGIN_M: f64 = 100.0;

/// A state with its own ALKIS service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Region {
    Brandenburg,
    Berlin,
    Saxony,
    SaxonyAnhalt,
    Hamburg,
    Hessen,
    SchleswigHolstein,
    MecklenburgVorpommern,
}

/// Who serves a region's rasters (orthophoto, terrain and surface models).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    /// LGB Brandenburg, also for Berlin.
    Lgb,
    /// GeoSN, Saxony.
    GeoSn,
    /// LVermGeo, Saxony-Anhalt (EPSG:25832).
    LvermGeo,
    /// LGV, Hamburg (EPSG:25832).
    Lgv,
    /// HVBG, Hesse (EPSG:25832).
    Hvbg,
    /// LVermGeo SH, Schleswig-Holstein (EPSG:25832).
    LvermGeoSh,
    /// LAiV M-V, Mecklenburg-Vorpommern (EPSG:25833, the first per-area provider in zone 33).
    LaivMv,
}

impl Provider {
    /// Whether the provider's rasters are fetched per map area (WMS, WCS, zip members) instead of
    /// as recorded tiles.
    pub fn per_area(self) -> bool {
        matches!(
            self,
            Self::LvermGeo | Self::Lgv | Self::Hvbg | Self::LvermGeoSh | Self::LaivMv
        )
    }
}

impl Region {
    /// All regions, in the order they are listed and queried.
    pub const ALL: [Self; 8] = [
        Self::Brandenburg,
        Self::Berlin,
        Self::Saxony,
        Self::SaxonyAnhalt,
        Self::Hamburg,
        Self::Hessen,
        Self::SchleswigHolstein,
        Self::MecklenburgVorpommern,
    ];

    /// The state an ALKIS object id belongs to (`DEBB…`, `DEBE…`, `DESN…`, `DEHE…`).
    pub fn of_object_id(oid: &str) -> Option<Self> {
        match oid.get(..4) {
            Some("DEBB") => Some(Self::Brandenburg),
            Some("DEBE") => Some(Self::Berlin),
            Some("DESN") => Some(Self::Saxony),
            Some("DEST") => Some(Self::SaxonyAnhalt),
            Some("DEHH") => Some(Self::Hamburg),
            Some("DEHE") => Some(Self::Hessen),
            Some("DESH") => Some(Self::SchleswigHolstein),
            Some("DEMV") => Some(Self::MecklenburgVorpommern),
            _ => None,
        }
    }

    pub fn provider(self) -> Provider {
        match self {
            Self::Brandenburg | Self::Berlin => Provider::Lgb,
            Self::Saxony => Provider::GeoSn,
            Self::SaxonyAnhalt => Provider::LvermGeo,
            Self::Hamburg => Provider::Lgv,
            Self::Hessen => Provider::Hvbg,
            Self::SchleswigHolstein => Provider::LvermGeoSh,
            Self::MecklenburgVorpommern => Provider::LaivMv,
        }
    }

    /// The CRS the region's rasters come in, and so the CRS of maps whose centre lies in it.
    pub fn crs(self) -> crate::crs::Crs {
        match self {
            Self::SaxonyAnhalt | Self::Hamburg | Self::Hessen | Self::SchleswigHolstein => {
                crate::crs::Crs::Utm32
            }
            _ => crate::crs::Crs::Utm33,
        }
    }

    /// The region's boundary, parsed once.
    fn boundary(self) -> &'static MultiPolygon<f64> {
        static BOUNDARIES: [OnceLock<MultiPolygon<f64>>; 8] = [
            OnceLock::new(),
            OnceLock::new(),
            OnceLock::new(),
            OnceLock::new(),
            OnceLock::new(),
            OnceLock::new(),
            OnceLock::new(),
            OnceLock::new(),
        ];
        let (index, text) = match self {
            Self::Brandenburg => (0, BRANDENBURG_BOUNDARY),
            Self::Berlin => (1, BERLIN_BOUNDARY),
            Self::Saxony => (2, SAXONY_BOUNDARY),
            Self::SaxonyAnhalt => (3, SAXONY_ANHALT_BOUNDARY),
            Self::Hamburg => (4, HAMBURG_BOUNDARY),
            Self::Hessen => (5, HESSEN_BOUNDARY),
            Self::SchleswigHolstein => (6, SCHLESWIG_HOLSTEIN_BOUNDARY),
            Self::MecklenburgVorpommern => (7, MECKLENBURG_VORPOMMERN_BOUNDARY),
        };
        BOUNDARIES[index].get_or_init(|| parse_boundary(text))
    }
}

/// The regions whose ALKIS data covers `bbox` (EPSG:25833), in [`Region::ALL`] order.
pub fn regions_for(bbox: &BoundingBox) -> Vec<Region> {
    let rect = rectangle(&bbox.buffered(MARGIN_M));
    Region::ALL
        .into_iter()
        .filter(|region| region.boundary().intersects(&rect))
        .collect()
}

/// Whether the centre of the 1 km zone-32 tile (EPSG:25832) lies in or near `region`, so its
/// provider must have the tile.
pub fn tile_centre_in(tile: TileId, region: Region) -> bool {
    let centre = crate::crs::ProjectedPoint {
        easting: f64::from(tile.east_km) * 1000.0 + 500.0,
        northing: f64::from(tile.north_km) * 1000.0 + 500.0,
    };
    crate::crs::Crs::Utm33
        .reproject(crate::crs::Crs::Utm32, centre)
        .is_some_and(|p| {
            regions_for(&BoundingBox {
                min_e: p.easting,
                min_n: p.northing,
                max_e: p.easting,
                max_n: p.northing,
            })
            .contains(&region)
        })
}

/// Whether the 1 km zone-32 tile (EPSG:25832) touches `region` at all: every tile that can
/// exist for it (a provider delivers whole or clipped tiles for each square that intersects).
pub fn tile_touches(tile: TileId, region: Region) -> bool {
    let corner = |de: u32, dn: u32| {
        crate::crs::Crs::Utm33.reproject(
            crate::crs::Crs::Utm32,
            crate::crs::ProjectedPoint {
                easting: f64::from(tile.east_km + de) * 1000.0,
                northing: f64::from(tile.north_km + dn) * 1000.0,
            },
        )
    };
    let corners = [corner(0, 0), corner(1, 0), corner(1, 1), corner(0, 1)];
    let Some(corners) = corners.into_iter().collect::<Option<Vec<_>>>() else {
        return false;
    };
    let ring: Vec<_> = corners
        .iter()
        .map(|p| coord! { x: p.easting, y: p.northing })
        .collect();
    let square = Polygon::new(geo::LineString::new(ring), Vec::new());
    region.boundary().intersects(&square)
}

/// The region whose rasters a map of `bbox` uses: the one containing its centre.
pub fn raster_region(bbox: &BoundingBox) -> Option<Region> {
    let centre = Point::new(
        (bbox.min_e + bbox.max_e) / 2.0,
        (bbox.min_n + bbox.max_n) / 2.0,
    );
    if !centre.x().is_finite() || !centre.y().is_finite() {
        return None;
    }
    Region::ALL
        .into_iter()
        .find(|region| region.boundary().contains(&centre))
        // The boundaries are simplified separately, so a few metres along a border belong to no
        // region: take a region within the margin.
        .or_else(|| {
            let point = BoundingBox {
                min_e: centre.x(),
                min_n: centre.y(),
                max_e: centre.x(),
                max_n: centre.y(),
            };
            regions_for(&point).into_iter().next()
        })
}

/// The tiles of a `km` grid over `bbox` that touch a region of `provider`; providers publish only
/// those.
pub fn provider_tiles(bbox: &BoundingBox, provider: Provider, km: u32) -> Vec<TileId> {
    // Per-area providers have no 25833 tile grid (and Hesse reaches negative eastings, which a
    // `TileId` cannot hold).
    if provider.per_area() {
        return Vec::new();
    }
    crate::tiles::tiles_covering_grid(bbox, km)
        .into_iter()
        .filter(|tile| {
            let rect = rectangle(&tile.bounds_km(km));
            Region::ALL
                .into_iter()
                .filter(|region| region.provider() == provider)
                .any(|region| region.boundary().intersects(&rect))
        })
        .collect()
}

/// Whether `bbox` itself (without the fetch margin) overlaps Berlin.
pub fn touches_berlin(bbox: &BoundingBox) -> bool {
    Region::Berlin.boundary().intersects(&rectangle(bbox))
}

fn rectangle(bbox: &BoundingBox) -> Polygon<f64> {
    Rect::new(
        coord! { x: bbox.min_e, y: bbox.min_n },
        coord! { x: bbox.max_e, y: bbox.max_n },
    )
    .to_polygon()
}

fn parse_boundary(text: &str) -> MultiPolygon<f64> {
    let collection: geojson::FeatureCollection =
        text.parse().expect("embedded boundary is GeoJSON");
    let geometry = collection.features[0]
        .geometry
        .clone()
        .expect("embedded boundary has a geometry");
    MultiPolygon::try_from(geometry.value).expect("embedded boundary is a MultiPolygon")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn around(e: f64, n: f64, half: f64) -> BoundingBox {
        BoundingBox {
            min_e: e - half,
            min_n: n - half,
            max_e: e + half,
            max_n: n + half,
        }
    }

    #[test]
    fn strausberg_is_brandenburg() {
        assert_eq!(
            regions_for(&around(421_582.0, 5_821_278.0, 500.0)),
            vec![Region::Brandenburg]
        );
    }

    #[test]
    fn mitte_is_berlin() {
        assert_eq!(
            regions_for(&around(391_000.0, 5_820_000.0, 500.0)),
            vec![Region::Berlin]
        );
    }

    #[test]
    fn border_needs_both() {
        // The southernmost boundary vertex, at Schönefeld.
        assert_eq!(
            regions_for(&around(407_850.0, 5_799_524.0, 300.0)),
            vec![Region::Brandenburg, Region::Berlin]
        );
    }

    #[test]
    fn margin_reaches_across_the_border() {
        // 50 m south of the boundary: still within the margin.
        assert_eq!(
            regions_for(&around(407_850.0, 5_799_424.0, 50.0)),
            vec![Region::Brandenburg, Region::Berlin]
        );
        // 300 m south: Brandenburg only.
        assert_eq!(
            regions_for(&around(407_850.0, 5_799_124.0, 50.0)),
            vec![Region::Brandenburg]
        );
    }

    #[test]
    fn dresden_is_saxony() {
        let school = around(415_973.0, 5_655_745.0, 300.0);
        assert_eq!(regions_for(&school), vec![Region::Saxony]);
        assert_eq!(raster_region(&school), Some(Region::Saxony));
        assert_eq!(Region::Saxony.provider(), Provider::GeoSn);
    }

    #[test]
    fn lausitz_border_needs_both() {
        // A vertex shared by the Brandenburg and Saxony boundaries.
        assert_eq!(
            regions_for(&around(459_509.0, 5_710_017.0, 300.0)),
            vec![Region::Brandenburg, Region::Saxony]
        );
    }

    #[test]
    fn goerlitz_east_edge_is_covered() {
        assert_eq!(
            raster_region(&around(501_000.0, 5_676_000.0, 300.0)),
            Some(Region::Saxony)
        );
    }

    #[test]
    fn berlin_hole_is_not_brandenburg() {
        let mitte = around(391_000.0, 5_820_000.0, 500.0);
        assert!(!regions_for(&mitte).contains(&Region::Brandenburg));
        assert_eq!(raster_region(&mitte), Some(Region::Berlin));
        assert_eq!(Region::Berlin.provider(), Provider::Lgb);
        assert_eq!(Region::Brandenburg.provider(), Provider::Lgb);
    }

    #[test]
    fn raster_region_follows_the_centre() {
        // Centred 50 m south of Berlin's southernmost vertex: Brandenburg, although the box
        // reaches into Berlin.
        let south = around(407_850.0, 5_799_474.0, 200.0);
        assert!(regions_for(&south).contains(&Region::Berlin));
        assert_eq!(raster_region(&south), Some(Region::Brandenburg));
        let north = around(407_850.0, 5_799_624.0, 200.0);
        assert_eq!(raster_region(&north), Some(Region::Berlin));
    }

    #[test]
    fn outside_every_region_is_refused() {
        // Lower Saxony, west of Saxony-Anhalt.
        let elsewhere = around(150_000.0, 5_800_000.0, 300.0);
        assert_eq!(raster_region(&elsewhere), None);
        assert!(regions_for(&elsewhere).is_empty());
        let nan = around(f64::NAN, 5_800_000.0, 300.0);
        assert_eq!(raster_region(&nan), None);
    }

    #[test]
    fn centre_in_a_boundary_sliver_has_a_region() {
        // Points that the separately simplified boundaries leave in no region.
        for (e, n) in [(391_595.0, 5_834_335.0), (418_969.0, 5_692_390.0)] {
            assert!(raster_region(&around(e, n, 100.0)).is_some(), "{e} {n}");
        }
    }

    #[test]
    fn provider_tiles_skip_tiles_outside_its_regions() {
        // Centre in Brandenburg 150 m north of the 1 km line; the tile south of it lies in Saxony.
        let border = around(410_500.0, 5_691_150.0, 300.0);
        let at = |east_km, north_km| crate::tiles::TileId { east_km, north_km };
        assert_eq!(
            crate::tiles::tiles_covering(&border),
            [at(410, 5690), at(410, 5691)]
        );
        assert_eq!(provider_tiles(&border, Provider::Lgb, 1), [at(410, 5691)]);
        assert_eq!(provider_tiles(&border, Provider::GeoSn, 2), [at(410, 5690)]);
    }

    #[test]
    fn halle_is_saxony_anhalt_in_zone_32() {
        // Latina August Hermann Francke, Halle (25833).
        let latina = around(289_795.0, 5_707_067.0, 200.0);
        assert_eq!(regions_for(&latina), vec![Region::SaxonyAnhalt]);
        assert_eq!(raster_region(&latina), Some(Region::SaxonyAnhalt));
        assert_eq!(Region::SaxonyAnhalt.provider(), Provider::LvermGeo);
        assert_eq!(Region::SaxonyAnhalt.crs(), crate::crs::Crs::Utm32);
        assert_eq!(Region::Saxony.crs(), crate::crs::Crs::Utm33);
        assert_eq!(
            Region::of_object_id("DESTLIKA0001MuQqBL"),
            Some(Region::SaxonyAnhalt)
        );
    }

    #[test]
    fn eppendorf_is_hamburg_in_zone_32() {
        // Gymnasium Eppendorf (25833).
        let school = around(168_078.0, 5_949_079.0, 200.0);
        assert_eq!(regions_for(&school), vec![Region::Hamburg]);
        assert_eq!(raster_region(&school), Some(Region::Hamburg));
        assert_eq!(Region::Hamburg.provider(), Provider::Lgv);
        assert_eq!(Region::Hamburg.crs(), crate::crs::Crs::Utm32);
        assert_eq!(
            Region::of_object_id("DEHHALKAP0007dHTBL"),
            Some(Region::Hamburg)
        );
    }

    #[test]
    fn frankfurt_school_is_hessen_in_zone_32() {
        // Lessing-Gymnasium Frankfurt (25833).
        let school = around(47_838.0, 5_571_740.0, 200.0);
        assert_eq!(regions_for(&school), vec![Region::Hessen]);
        assert_eq!(raster_region(&school), Some(Region::Hessen));
        assert_eq!(Region::Hessen.provider(), Provider::Hvbg);
        assert!(Provider::Hvbg.per_area());
        assert_eq!(Region::Hessen.crs(), crate::crs::Crs::Utm32);
        assert_eq!(crate::project::map_crs(&school), crate::crs::Crs::Utm32);
    }

    #[test]
    fn wiesbaden_with_negative_eastings_is_hessen() {
        // West of the 25833 central meridian's origin: eastings below zero.
        let west = around(-5_000.0, 5_564_000.0, 200.0);
        assert_eq!(regions_for(&west), vec![Region::Hessen]);
        assert_eq!(raster_region(&west), Some(Region::Hessen));
        assert!(!touches_berlin(&west));
        // A per-area provider never has a 25833 tile grid.
        assert!(provider_tiles(&west, Provider::Hvbg, 1).is_empty());
        assert!(provider_tiles(&west, Provider::Hvbg, 2).is_empty());
    }

    #[test]
    fn hessen_object_ids() {
        assert_eq!(
            Region::of_object_id("DEHE0000AbCdEfGh"),
            Some(Region::Hessen)
        );
        assert_eq!(
            Region::of_object_id("DEHH0000AbCdEfGh"),
            Some(Region::Hamburg)
        );
    }

    #[test]
    fn norderstedt_school_is_schleswig_holstein_in_zone_32() {
        // Coppernicus-Gymnasium Norderstedt (OSM way/26986780), 25832 E 565214 / N 5949250.
        let school = around(169_108.0, 5_960_473.0, 200.0);
        assert_eq!(regions_for(&school), vec![Region::SchleswigHolstein]);
        assert_eq!(raster_region(&school), Some(Region::SchleswigHolstein));
        assert_eq!(Region::SchleswigHolstein.provider(), Provider::LvermGeoSh);
        assert!(Provider::LvermGeoSh.per_area());
        assert_eq!(Region::SchleswigHolstein.crs(), crate::crs::Crs::Utm32);
        assert_eq!(crate::project::map_crs(&school), crate::crs::Crs::Utm32);
        assert!(provider_tiles(&school, Provider::LvermGeoSh, 1).is_empty());
    }

    #[test]
    fn straddling_area_has_both_regions() {
        // 25832 E 565500 / N 5947700, on the Norderstedt / Hamburg-Langenhorn border.
        let to_33 = |e: f64, n: f64| {
            crate::crs::Crs::Utm33
                .reproject(
                    crate::crs::Crs::Utm32,
                    crate::crs::ProjectedPoint {
                        easting: e,
                        northing: n,
                    },
                )
                .unwrap()
        };
        // Centre in Schleswig-Holstein (north of the border), box reaching into Hamburg.
        let c = to_33(565_500.0, 5_948_100.0);
        let north = around(c.easting, c.northing, 700.0);
        assert_eq!(
            regions_for(&north),
            vec![Region::Hamburg, Region::SchleswigHolstein]
        );
        assert_eq!(raster_region(&north), Some(Region::SchleswigHolstein));
        // Centre in Hamburg (south of the border), box reaching into Schleswig-Holstein.
        let c = to_33(565_500.0, 5_947_100.0);
        let south = around(c.easting, c.northing, 700.0);
        assert_eq!(
            regions_for(&south),
            vec![Region::Hamburg, Region::SchleswigHolstein]
        );
        assert_eq!(raster_region(&south), Some(Region::Hamburg));
    }

    #[test]
    fn tile_touches_inside_on_the_border_and_outside() {
        let at = |east_km, north_km| TileId { east_km, north_km };
        // Hamburg-Langenhorn, well inside; Norderstedt, well inside Schleswig-Holstein.
        assert!(tile_touches(at(567, 5944), Region::Hamburg));
        assert!(!tile_touches(at(567, 5944), Region::SchleswigHolstein));
        assert!(tile_touches(at(565, 5951), Region::SchleswigHolstein));
        assert!(!tile_touches(at(565, 5951), Region::Hamburg));
        // On the border: 565/5948 is mostly Norderstedt but touches Hamburg; 566/5947 is mostly
        // Hamburg but 566/5948 reaches into Schleswig-Holstein.
        assert!(tile_touches(at(565, 5948), Region::Hamburg));
        assert!(tile_touches(at(565, 5948), Region::SchleswigHolstein));
        assert!(tile_touches(at(566, 5948), Region::SchleswigHolstein));
        // Inside Hamburg just south of the border: 566/5947 does not reach Schleswig-Holstein.
        assert!(!tile_touches(at(566, 5947), Region::SchleswigHolstein));
        // Far from both states (E 400 km, N 5000 km): it touches neither.
        assert!(!tile_touches(at(400, 5000), Region::Hamburg));
        assert!(!tile_touches(at(400, 5000), Region::SchleswigHolstein));
    }

    #[test]
    fn langenhorn_stays_hamburg() {
        // A point in Hamburg-Langenhorn, 25832 E 566000 / N 5946500, about 2 km south of the border.
        let p = crate::crs::Crs::Utm33
            .reproject(
                crate::crs::Crs::Utm32,
                crate::crs::ProjectedPoint {
                    easting: 566_000.0,
                    northing: 5_946_500.0,
                },
            )
            .unwrap();
        let langenhorn = around(p.easting, p.northing, 200.0);
        assert_eq!(regions_for(&langenhorn), vec![Region::Hamburg]);
        assert_eq!(raster_region(&langenhorn), Some(Region::Hamburg));
    }

    #[test]
    fn schleswig_holstein_object_ids() {
        assert_eq!(
            Region::of_object_id("DESH0000AbCdEfGh"),
            Some(Region::SchleswigHolstein)
        );
        assert_eq!(
            Region::of_object_id("DEHH0000AbCdEfGh"),
            Some(Region::Hamburg)
        );
    }

    #[test]
    fn rostock_school_is_mecklenburg_vorpommern_in_zone_33() {
        // Innerstaedtisches Gymnasium Rostock (OSM way/402355834), 25833 E 312157 / N 5996317.
        let school = around(312_157.0, 5_996_317.0, 200.0);
        assert_eq!(regions_for(&school), vec![Region::MecklenburgVorpommern]);
        assert_eq!(raster_region(&school), Some(Region::MecklenburgVorpommern));
        assert_eq!(Region::MecklenburgVorpommern.provider(), Provider::LaivMv);
        assert!(Provider::LaivMv.per_area());
        assert_eq!(Region::MecklenburgVorpommern.crs(), crate::crs::Crs::Utm33);
        assert_eq!(crate::project::map_crs(&school), crate::crs::Crs::Utm33);
        assert!(provider_tiles(&school, Provider::LaivMv, 1).is_empty());
        // The same place in zone 32 (E 704637 / N 5997030) is the same point.
        let p = crate::crs::Crs::Utm32
            .reproject(
                crate::crs::Crs::Utm33,
                crate::crs::ProjectedPoint {
                    easting: 312_157.0,
                    northing: 5_996_317.0,
                },
            )
            .unwrap();
        assert!((p.easting - 704_637.0).abs() < 2.0 && (p.northing - 5_997_030.0).abs() < 2.0);
    }

    #[test]
    fn brussow_border_has_both_regions() {
        // Bruessow on the Brandenburg/MV border, 25833 E 436007-438007, N 5917677-5919677.
        let border = BoundingBox {
            min_e: 436_007.0,
            min_n: 5_917_677.0,
            max_e: 438_007.0,
            max_n: 5_919_677.0,
        };
        let regions = regions_for(&border);
        assert!(regions.contains(&Region::Brandenburg));
        assert!(regions.contains(&Region::MecklenburgVorpommern));
        // The raster region follows the centre; both sides stay in zone 33.
        let centre = raster_region(&border).unwrap();
        assert!(matches!(
            centre,
            Region::Brandenburg | Region::MecklenburgVorpommern
        ));
        assert_eq!(crate::project::map_crs(&border), crate::crs::Crs::Utm33);
        // A map centred well inside either side takes that side's provider.
        let north = around(437_007.0, 5_919_400.0, 100.0);
        let south = around(437_007.0, 5_917_900.0, 100.0);
        let providers: Vec<_> = [north, south]
            .iter()
            .map(|b| raster_region(b).unwrap().provider())
            .collect();
        assert!(providers.contains(&Provider::LaivMv));
        assert!(providers.contains(&Provider::Lgb));
    }

    #[test]
    fn mv_object_ids() {
        assert_eq!(
            Region::of_object_id("DEMV0000AbCdEfGh"),
            Some(Region::MecklenburgVorpommern)
        );
        assert_eq!(
            Region::of_object_id("DEBB0000AbCdEfGh"),
            Some(Region::Brandenburg)
        );
    }

    #[test]
    fn object_ids_name_their_state() {
        assert_eq!(
            Region::of_object_id("DEBE03YY600004AR"),
            Some(Region::Berlin)
        );
        assert_eq!(
            Region::of_object_id("DEBBAL0100000001"),
            Some(Region::Brandenburg)
        );
        assert_eq!(
            Region::of_object_id("DESNALK0CrM0003UBL"),
            Some(Region::Saxony)
        );
        assert_eq!(Region::of_object_id("DETH01"), None);
    }

    #[test]
    fn touching_berlin_needs_no_margin() {
        // 50 m south of the boundary: within the fetch margin, but not in Berlin.
        assert!(!touches_berlin(&around(407_850.0, 5_799_424.0, 40.0)));
        assert!(touches_berlin(&around(407_850.0, 5_799_524.0, 40.0)));
    }

    #[test]
    fn boundary_has_all_parts() {
        // After the validity repair (tests/region_assets.rs): an overlapping fragment merged into
        // the main polygon, slivers under 100 m² dropped.
        assert_eq!(Region::Berlin.boundary().0.len(), 2);
        assert_eq!(Region::Berlin.boundary().0[0].exterior().0.len(), 684);
        assert_eq!(
            Region::Brandenburg.boundary().0[0].interiors().len(),
            3,
            "Berlin and small enclaves"
        );
    }
}
