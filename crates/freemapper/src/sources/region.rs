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
/// North Rhine-Westphalia from Geobasis NRW ALKIS vereinfacht `VerwaltungsEinheit` DE05
/// (simplified 25 m, EPSG:25833; dl-de/zero-2-0), with the five Vennbahn exclaves. In EPSG:25833 it
/// has negative eastings (from about -140 km to 119 km).
const NORDRHEIN_WESTFALEN_BOUNDARY: &str =
    include_str!("../../../../assets/regions/nordrhein-westfalen.geojson");
/// Lower Saxony from LGLN ALKIS vereinfacht `VerwaltungsEinheit` DE03 (simplified 25 m, EPSG:25833;
/// CC BY 4.0). Bremen and Bremerhaven are holes: they are not Niedersachsen.
const NIEDERSACHSEN_BOUNDARY: &str =
    include_str!("../../../../assets/regions/niedersachsen.geojson");
/// Rhineland-Palatinate from the LVermGeo RP WFS `Verwaltungsgrenzen` (simplified 25 m, EPSG:25833;
/// dl-de/by-2-0). In EPSG:25833 it has negative eastings (84.5 % of the state). Wiesbaden and
/// Mainz-Kastel are Hesse, not part of it.
const RHEINLAND_PFALZ_BOUNDARY: &str =
    include_str!("../../../../assets/regions/rheinland-pfalz.geojson");
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
    NordrheinWestfalen,
    Niedersachsen,
    RheinlandPfalz,
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
    /// Geobasis NRW (Bezirksregierung Köln), North Rhine-Westphalia (EPSG:25832).
    GeobasisNrw,
    /// LGLN, Niedersachsen (EPSG:25832).
    Lgln,
    /// LVermGeo Rheinland-Pfalz (EPSG:25832).
    LvermGeoRp,
}

impl Provider {
    /// The terrain model the contours come from, for `freemapper:source` on contour candidates.
    pub fn terrain_source(self) -> &'static str {
        match self {
            Self::Lgb => "lgb-dgm1",
            Self::GeoSn => "geosn-dgm1",
            Self::LvermGeo => "lvermgeo-st-dgm1",
            Self::Lgv => "lgv-dgm1",
            Self::Hvbg => "hvbg-dgm1",
            Self::LvermGeoSh => "lvermgeo-sh-dgm1",
            Self::LaivMv => "laiv-mv-dgm1",
            Self::GeobasisNrw => "geobasis-nrw-dgm1",
            Self::Lgln => "lgln-dgm1",
            Self::LvermGeoRp => "lvermgeo-rp-dgm1",
        }
    }

    /// Whether the provider's rasters are fetched per map area (WMS, WCS, zip members) instead of
    /// as recorded tiles.
    pub fn per_area(self) -> bool {
        matches!(
            self,
            Self::LvermGeo
                | Self::Lgv
                | Self::Hvbg
                | Self::LvermGeoSh
                | Self::LaivMv
                | Self::GeobasisNrw
                | Self::Lgln
                | Self::LvermGeoRp
        )
    }
}

impl Region {
    /// All regions, in the order they are listed and queried.
    pub const ALL: [Self; 11] = [
        Self::Brandenburg,
        Self::Berlin,
        Self::Saxony,
        Self::SaxonyAnhalt,
        Self::Hamburg,
        Self::Hessen,
        Self::SchleswigHolstein,
        Self::MecklenburgVorpommern,
        Self::NordrheinWestfalen,
        Self::Niedersachsen,
        Self::RheinlandPfalz,
    ];

    /// The state an ALKIS object id belongs to, by its first four characters (`DEBB…`, `DEBE…`,
    /// `DESN…`, `DEST…`, `DEHH…`, `DEHE…`, `DESH…`, `DEMV…`, `DENW…`, `DENI…`, `DERP…`).
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
            Some("DENW") => Some(Self::NordrheinWestfalen),
            Some("DENI") => Some(Self::Niedersachsen),
            Some("DERP") => Some(Self::RheinlandPfalz),
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
            Self::NordrheinWestfalen => Provider::GeobasisNrw,
            Self::Niedersachsen => Provider::Lgln,
            Self::RheinlandPfalz => Provider::LvermGeoRp,
        }
    }

    /// The CRS the region's rasters come in, and so the CRS of maps whose centre lies in it.
    pub fn crs(self) -> crate::crs::Crs {
        match self {
            Self::SaxonyAnhalt
            | Self::Hamburg
            | Self::Hessen
            | Self::SchleswigHolstein
            | Self::NordrheinWestfalen
            | Self::Niedersachsen
            | Self::RheinlandPfalz => crate::crs::Crs::Utm32,
            _ => crate::crs::Crs::Utm33,
        }
    }

    /// The region's boundary, parsed once.
    fn boundary(self) -> &'static MultiPolygon<f64> {
        static BOUNDARIES: [OnceLock<MultiPolygon<f64>>; 11] = [
            OnceLock::new(),
            OnceLock::new(),
            OnceLock::new(),
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
            Self::NordrheinWestfalen => (8, NORDRHEIN_WESTFALEN_BOUNDARY),
            Self::Niedersachsen => (9, NIEDERSACHSEN_BOUNDARY),
            Self::RheinlandPfalz => (10, RHEINLAND_PFALZ_BOUNDARY),
        };
        BOUNDARIES[index].get_or_init(|| parse_boundary(text))
    }
}

/// The regions whose ALKIS data covers `bbox` (EPSG:25833), in [`Region::ALL`] order.
pub fn regions_for(bbox: &BoundingBox) -> Vec<Region> {
    Region::ALL
        .into_iter()
        .filter(|region| within_margin(region.boundary(), bbox))
        .collect()
}

/// Whether `boundary` (EPSG:25833) lies within [`MARGIN_M`] of `bbox`: the rule by which an area
/// needs a region, and an OSM extract.
pub(crate) fn within_margin(boundary: &MultiPolygon<f64>, bbox: &BoundingBox) -> bool {
    boundary.intersects(&rectangle(&bbox.buffered(MARGIN_M)))
}

/// Whether the point (EPSG:25833) lies inside `region`'s boundary itself (no margin).
pub fn contains_point(region: Region, easting: f64, northing: f64) -> bool {
    region.boundary().contains(&Point::new(easting, northing))
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

/// How far a point must lie from every supported region to count as outside them all. The
/// boundaries are simplified separately (up to 25 m each), so a shared border can leave a gap of
/// up to about 50 m between two states.
const UNCOVERED_M: f64 = 50.0;
/// The spacing of the points [`reaches_beyond_coverage`] tests, half of [`UNCOVERED_M`].
const COVERAGE_STEP_M: f64 = 25.0;

/// Whether part of `bbox` (EPSG:25833) lies outside every supported region: a point of it more
/// than [`UNCOVERED_M`] from all their boundaries, e.g. in Bremen (a hole in Niedersachsen), in
/// Thuringia or another state that is no region yet, abroad or at sea.
pub fn reaches_beyond_coverage(bbox: &BoundingBox) -> bool {
    use geo::{BooleanOps as _, Distance as _, Euclidean};
    // Every boundary part within UNCOVERED_M of a point of `bbox` lies in this rectangle.
    let near = rectangle(&bbox.buffered(UNCOVERED_M));
    let parts: Vec<MultiPolygon<f64>> = Region::ALL
        .into_iter()
        .filter(|region| region.boundary().intersects(&near))
        .map(|region| region.boundary().intersection(&near))
        .collect();
    let steps = |min: f64, max: f64| {
        let n = ((max - min) / COVERAGE_STEP_M).ceil().max(1.0) as u32;
        (0..=n).map(move |i| (min + f64::from(i) * COVERAGE_STEP_M).min(max))
    };
    steps(bbox.min_e, bbox.max_e).any(|e| {
        steps(bbox.min_n, bbox.max_n).any(|n| {
            let point = Point::new(e, n);
            !parts.iter().any(|part| part.contains(&point))
                && parts
                    .iter()
                    .all(|part| Euclidean.distance(&point, part) > UNCOVERED_M)
        })
    })
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

/// The MultiPolygon of the first feature of an embedded GeoJSON boundary asset.
pub(crate) fn parse_boundary(text: &str) -> MultiPolygon<f64> {
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
    #[test]
    fn each_provider_names_its_own_terrain_source() {
        use super::Provider::*;
        let all = [
            Lgb,
            GeoSn,
            LvermGeo,
            Lgv,
            Hvbg,
            LvermGeoSh,
            LaivMv,
            GeobasisNrw,
            Lgln,
            LvermGeoRp,
        ];
        let names: Vec<&str> = all.iter().map(|p| p.terrain_source()).collect();
        assert_eq!(
            names,
            [
                "lgb-dgm1",
                "geosn-dgm1",
                "lvermgeo-st-dgm1",
                "lgv-dgm1",
                "hvbg-dgm1",
                "lvermgeo-sh-dgm1",
                "laiv-mv-dgm1",
                "geobasis-nrw-dgm1",
                "lgln-dgm1",
                "lvermgeo-rp-dgm1",
            ]
        );
    }

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
        // Poland, east of Brandenburg.
        let elsewhere = around(600_000.0, 5_800_000.0, 300.0);
        assert_eq!(raster_region(&elsewhere), None);
        assert!(regions_for(&elsewhere).is_empty());
        let nan = around(f64::NAN, 5_800_000.0, 300.0);
        assert_eq!(raster_region(&nan), None);
    }

    #[test]
    fn coverage_gaps_are_found_beyond_the_supported_states_only() {
        // Bremen probe, tile 32 485 5876 (Stuhr-Brinkum / Bremen-Huchting, 44 % Bremen).
        assert!(reaches_beyond_coverage(&around(
            83_417.0,
            5_894_566.0,
            300.0
        )));
        // Poland, just east of the Oder at Frankfurt (Oder).
        assert!(reaches_beyond_coverage(&around(
            474_000.0,
            5_799_000.0,
            600.0
        )));
        // Inside one state.
        assert!(!reaches_beyond_coverage(&around(
            412_000.0,
            5_820_000.0,
            2000.0
        )));
        // Shared borders between supported states, including the slivers the separately
        // simplified boundaries leave in no region.
        for (e, n) in [
            (391_595.0, 5_834_335.0),
            (418_969.0, 5_692_390.0),
            (410_500.0, 5_691_150.0),
        ] {
            assert!(!reaches_beyond_coverage(&around(e, n, 600.0)), "{e} {n}");
        }
        // Lower Saxony's borders with supported states, at points where the boundaries meet.
        for (border, e, n) in [
            ("NI/NRW, 52.18 N 7.93 E", 16_879.0, 5_804_658.0),
            ("NI/Saxony-Anhalt at Helmstedt", 231_964.0, 5_792_768.0),
            ("NI/Hesse, 51.55 N 9.61 E", 126_608.0, 5_724_755.0),
            (
                "NI/Schleswig-Holstein in the Elbe at Glückstadt",
                129_966.0,
                5_975_154.0,
            ),
            (
                "NI/Mecklenburg-Vorpommern in the Elbe at Dömitz",
                249_166.0,
                5_893_748.0,
            ),
            (
                "NI/Mecklenburg-Vorpommern near Boizenburg",
                220_264.0,
                5_915_541.0,
            ),
        ] {
            let area = around(e, n, 600.0);
            assert_eq!(regions_for(&area).len(), 2, "{border}");
            assert!(
                regions_for(&area).contains(&Region::Niedersachsen),
                "{border}"
            );
            assert!(!reaches_beyond_coverage(&area), "{border}");
        }
        // Lower Saxony/Thuringia east of Duderstadt: Thuringia is no region yet.
        assert!(reaches_beyond_coverage(&around(
            173_912.0,
            5_715_904.0,
            600.0
        )));
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
        assert_eq!(raster_region(&border), Some(Region::MecklenburgVorpommern));
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
    fn koeln_school_is_nrw_in_zone_32() {
        // Hildegard-von-Bingen-Gymnasium Koeln-Suelz (OSM way/56752699), 25833 E -66680 /
        // N 5671681, 25832 E 354704 / N 5642657 (tile 32 354 5642).
        let school = around(-66_680.0, 5_671_681.0, 200.0);
        assert_eq!(regions_for(&school), vec![Region::NordrheinWestfalen]);
        assert_eq!(raster_region(&school), Some(Region::NordrheinWestfalen));
        assert_eq!(Region::NordrheinWestfalen.provider(), Provider::GeobasisNrw);
        assert!(Provider::GeobasisNrw.per_area());
        assert_eq!(Region::NordrheinWestfalen.crs(), crate::crs::Crs::Utm32);
        assert_eq!(crate::project::map_crs(&school), crate::crs::Crs::Utm32);
        assert!(provider_tiles(&school, Provider::GeobasisNrw, 1).is_empty());
        let p = crate::crs::Crs::Utm32
            .reproject(
                crate::crs::Crs::Utm33,
                crate::crs::ProjectedPoint {
                    easting: -66_680.0,
                    northing: 5_671_681.0,
                },
            )
            .unwrap();
        assert!((p.easting - 354_704.0).abs() < 2.0 && (p.northing - 5_642_657.0).abs() < 2.0);
        assert_eq!(
            serde_json::to_string(&Region::NordrheinWestfalen).unwrap(),
            r#""nordrheinwestfalen""#
        );
        assert_eq!(
            serde_json::to_string(&Provider::GeobasisNrw).unwrap(),
            r#""geobasisnrw""#
        );
    }

    #[test]
    fn aachen_negative_eastings_is_nrw() {
        // Aachen cathedral (50.7753 N, 6.0839 E), 25833 E -128141 / N 5662838: far west of the
        // zone-33 origin.
        let aachen = around(-128_141.0, 5_662_838.0, 300.0);
        assert_eq!(regions_for(&aachen), vec![Region::NordrheinWestfalen]);
        assert_eq!(raster_region(&aachen), Some(Region::NordrheinWestfalen));
        assert!(contains_point(
            Region::NordrheinWestfalen,
            -128_141.0,
            5_662_838.0
        ));
        assert!(!touches_berlin(&aachen));
        assert_eq!(crate::project::map_crs(&aachen), crate::crs::Crs::Utm32);
        assert!(provider_tiles(&aachen, Provider::GeobasisNrw, 1).is_empty());
        assert!(provider_tiles(&aachen, Provider::GeobasisNrw, 2).is_empty());
        // Belgium, 7 km further west (E -135521): no region.
        let belgium = around(-135_521.0, 5_663_143.0, 300.0);
        assert!(regions_for(&belgium).is_empty());
        assert_eq!(raster_region(&belgium), None);
    }

    /// A box of half-width `half` m around a WGS84 point, in EPSG:25833.
    fn around_lonlat(lat: f64, lon: f64, half: f64) -> BoundingBox {
        let p = crate::crs::utm33_forward(crate::crs::LatLon { lat, lon });
        around(p.easting, p.northing, half)
    }

    #[test]
    fn hannover_school_is_niedersachsen_in_zone_32() {
        // Kaiser-Wilhelm- und Ratsgymnasium Hannover (OSM way/42796993), about 52.3749 N 9.7428 E.
        let school = around_lonlat(52.3749, 9.7428, 200.0);
        assert_eq!(regions_for(&school), vec![Region::Niedersachsen]);
        assert_eq!(raster_region(&school), Some(Region::Niedersachsen));
        assert_eq!(Region::Niedersachsen.provider(), Provider::Lgln);
        assert!(Provider::Lgln.per_area());
        assert_eq!(Region::Niedersachsen.crs(), crate::crs::Crs::Utm32);
        assert_eq!(crate::project::map_crs(&school), crate::crs::Crs::Utm32);
        assert!(provider_tiles(&school, Provider::Lgln, 1).is_empty());
        assert_eq!(
            serde_json::to_string(&Region::Niedersachsen).unwrap(),
            r#""niedersachsen""#
        );
        assert_eq!(serde_json::to_string(&Provider::Lgln).unwrap(), r#""lgln""#);
    }

    #[test]
    fn bremen_is_not_niedersachsen() {
        // Bremen market square and Bremerhaven town centre are holes in the Niedersachsen
        // boundary and have no region of their own yet.
        for (lat, lon) in [(53.0758, 8.8072), (53.5396, 8.5809)] {
            let area = around_lonlat(lat, lon, 300.0);
            assert!(regions_for(&area).is_empty(), "{lat} {lon}");
            assert_eq!(raster_region(&area), None);
            let p = crate::crs::utm33_forward(crate::crs::LatLon { lat, lon });
            assert!(!contains_point(
                Region::Niedersachsen,
                p.easting,
                p.northing
            ));
        }
        // Verden (an der Aller), 30 km south-east of Bremen, is Niedersachsen.
        let verden = around_lonlat(52.9227, 9.2287, 300.0);
        assert_eq!(regions_for(&verden), vec![Region::Niedersachsen]);
    }

    #[test]
    fn harburg_border_has_both_regions() {
        // Hamburg-Harburg town centre (53.46 N 9.98 E) lies in Hamburg; Seevetal, 6 km south,
        // in Niedersachsen. An area on the border touches both.
        let harburg = crate::crs::utm33_forward(crate::crs::LatLon {
            lat: 53.46,
            lon: 9.98,
        });
        let seevetal = crate::crs::utm33_forward(crate::crs::LatLon {
            lat: 53.40,
            lon: 9.98,
        });
        assert_eq!(
            regions_for(&around(harburg.easting, harburg.northing, 300.0)),
            vec![Region::Hamburg]
        );
        assert_eq!(
            regions_for(&around(seevetal.easting, seevetal.northing, 300.0)),
            vec![Region::Niedersachsen]
        );
        // Walk south from Harburg to the state border: the first box touching both regions has its
        // raster provider follow its centre.
        let mut found = false;
        for step in 0..120 {
            let n = harburg.northing - f64::from(step) * 50.0;
            let area = around(harburg.easting, n, 700.0);
            if regions_for(&area) == vec![Region::Hamburg, Region::Niedersachsen] {
                found = true;
                let centre_region = if contains_point(Region::Hamburg, harburg.easting, n) {
                    Region::Hamburg
                } else {
                    Region::Niedersachsen
                };
                assert_eq!(raster_region(&area), Some(centre_region));
            }
        }
        assert!(found, "no box on the Hamburg / Niedersachsen border");
    }

    #[test]
    fn ni_object_ids() {
        assert_eq!(
            Region::of_object_id("DENI0300AbCdEfGh"),
            Some(Region::Niedersachsen)
        );
        assert_eq!(
            Region::of_object_id("DENW37AL0000AbCd"),
            Some(Region::NordrheinWestfalen)
        );
    }

    #[test]
    fn mainz_school_is_rp_in_zone_32() {
        // Frauenlob-Gymnasium Mainz-Neustadt (OSM way/60400704), 50.00908 N 8.26574 E.
        let school = around_lonlat(50.00908, 8.26574, 200.0);
        assert_eq!(regions_for(&school), vec![Region::RheinlandPfalz]);
        assert_eq!(raster_region(&school), Some(Region::RheinlandPfalz));
        assert_eq!(Region::RheinlandPfalz.provider(), Provider::LvermGeoRp);
        assert!(Provider::LvermGeoRp.per_area());
        assert_eq!(Region::RheinlandPfalz.crs(), crate::crs::Crs::Utm32);
        assert_eq!(crate::project::map_crs(&school), crate::crs::Crs::Utm32);
        assert!(provider_tiles(&school, Provider::LvermGeoRp, 1).is_empty());
        assert_eq!(
            serde_json::to_string(&Region::RheinlandPfalz).unwrap(),
            r#""rheinlandpfalz""#
        );
        assert_eq!(
            serde_json::to_string(&Provider::LvermGeoRp).unwrap(),
            r#""lvermgeorp""#
        );
        // Koblenz, Goerres-Gymnasium: negative 25833 eastings.
        let koblenz = around_lonlat(50.35964, 7.59917, 200.0);
        assert_eq!(regions_for(&koblenz), vec![Region::RheinlandPfalz]);
    }

    #[test]
    fn wiesbaden_stays_hessen() {
        // Wiesbaden Hauptbahnhof and Mainz-Kastel (Hesse, east of the Rhine).
        for (lat, lon) in [(50.0710, 8.2437), (50.0072, 8.2886)] {
            let area = around_lonlat(lat, lon, 150.0);
            assert_eq!(regions_for(&area), vec![Region::Hessen], "{lat} {lon}");
            assert_eq!(raster_region(&area), Some(Region::Hessen));
            let p = crate::crs::utm33_forward(crate::crs::LatLon { lat, lon });
            assert!(contains_point(Region::Hessen, p.easting, p.northing));
            assert!(!contains_point(
                Region::RheinlandPfalz,
                p.easting,
                p.northing
            ));
        }
    }

    #[test]
    fn rp_object_ids() {
        assert_eq!(
            Region::of_object_id("DERPLP050000AR8IBL"),
            Some(Region::RheinlandPfalz)
        );
        assert_eq!(
            Region::of_object_id("DEHE0000AbCdEfGh"),
            Some(Region::Hessen)
        );
    }

    #[test]
    fn nrw_object_ids() {
        assert_eq!(
            Region::of_object_id("DENW37AL0000AbCd"),
            Some(Region::NordrheinWestfalen)
        );
        assert_eq!(
            Region::of_object_id("DEHE0000AbCdEfGh"),
            Some(Region::Hessen)
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
