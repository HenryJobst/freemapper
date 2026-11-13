//! Regions: the states freemapper has data for, and which of them cover an area.
//!
//! Every state keeps its own ALKIS service: the LGB service returns no features inside Berlin and
//! the Berlin service none outside, so an area near a border needs several. The rasters come
//! from one provider, the one of the region containing the area's centre (LGB also serves Berlin).

pub use freemapper_adapter_api::geometry::{
    contains_point_in, overlaps_in, provider_tiles_in, raster_region_in,
    reaches_beyond_coverage_in, regions_for_in, tile_centre_near, tile_touches_in,
    touches_region_in, within_margin,
};

use freemapper_adapter_api::rasters::RasterKind;
use freemapper_adapter_api::records::AdapterId;

use crate::adapters::{RegionAdapter, registry};
use crate::crs::BoundingBox;
use crate::tiles::TileId;

/// The adapter with the id `region` in the registry of the process.
fn adapter(region: &str) -> Option<&'static dyn RegionAdapter> {
    registry().get(region)
}

/// The regions whose ALKIS data covers `bbox` (EPSG:25833), by id, in the order of the registry.
pub fn regions_for(bbox: &BoundingBox) -> Vec<AdapterId> {
    regions_for_in(registry(), bbox)
        .into_iter()
        .map(|adapter| adapter.info().id.clone())
        .collect()
}

/// Whether the point (EPSG:25833) lies inside the boundary of `region` itself (no margin).
pub fn contains_point(region: impl AsRef<str>, easting: f64, northing: f64) -> bool {
    adapter(region.as_ref()).is_some_and(|adapter| contains_point_in(adapter, easting, northing))
}

/// Whether the centre of the 1 km zone-32 tile (EPSG:25832) lies in or near `region`, so its
/// provider must have the tile.
pub fn tile_centre_in(tile: TileId, region: impl AsRef<str>) -> bool {
    adapter(region.as_ref()).is_some_and(|adapter| tile_centre_near(adapter, tile))
}

/// Whether the 1 km zone-32 tile (EPSG:25832) touches `region` at all: every tile that can
/// exist for it (a provider delivers whole or clipped tiles for each square that intersects).
pub fn tile_touches(tile: TileId, region: impl AsRef<str>) -> bool {
    adapter(region.as_ref()).is_some_and(|adapter| tile_touches_in(adapter, tile))
}

/// The region whose rasters a map of `bbox` uses, by id: the one containing its centre.
pub fn raster_region(bbox: &BoundingBox) -> Option<AdapterId> {
    raster_region_in(registry(), bbox).map(|adapter| adapter.info().id.clone())
}

/// The provider of the rasters of a map of `bbox`, by id: that of its raster region.
pub fn raster_provider(bbox: &BoundingBox) -> Option<AdapterId> {
    raster_region_in(registry(), bbox).map(|adapter| adapter.info().provider.clone())
}

/// The terrain model the contours of `provider` come from, for `freemapper:source` on contour
/// candidates. For a provider the registry does not know (a project outside every region,
/// which holds no terrain model), that of the first provider of the registry, as it always was.
pub fn terrain_source(provider: Option<&AdapterId>) -> String {
    let backends = || {
        registry()
            .adapters()
            .filter_map(|adapter| adapter.rasters())
    };
    provider
        .and_then(|id| backends().find(|backend| backend.provider().id == *id))
        .or_else(|| backends().next())
        .map(|backend| backend.provider().terrain_source.clone())
        .unwrap_or_default()
}

/// The tiles of a `km` grid over `bbox` that touch a region of `provider`; providers publish only
/// those. None for a provider that fetches per area: it has no 25833 tile grid (and Hesse
/// reaches negative eastings, which a `TileId` cannot hold).
pub fn provider_tiles(bbox: &BoundingBox, provider: impl AsRef<str>, km: u32) -> Vec<TileId> {
    let provider = provider.as_ref();
    let per_area = registry()
        .adapters()
        .filter(|adapter| adapter.info().provider.as_str() == provider)
        .filter_map(|adapter| adapter.rasters())
        .any(|backend| backend.provider().kind == RasterKind::PerArea);
    if per_area {
        return Vec::new();
    }
    provider_tiles_in(registry(), bbox, provider, km)
}

/// Whether part of `bbox` (EPSG:25833) lies outside every supported region: a point of it more
/// than 50 m from all their boundaries.
pub fn reaches_beyond_coverage(bbox: &BoundingBox) -> bool {
    reaches_beyond_coverage_in(registry(), bbox)
}

/// The warnings of the regions `bbox` itself (without the fetch margin) lies on for a project
/// without buildings ([`RegionAdapter::no_buildings_warning`]), in registry order.
pub fn no_buildings_warnings(bbox: &BoundingBox) -> Vec<&'static str> {
    registry()
        .adapters()
        .filter(|adapter| overlaps_in(*adapter, bbox))
        .filter_map(|adapter| adapter.no_buildings_warning())
        .collect()
}

/// The region an ALKIS object id belongs to, by the prefix its adapter names (`DEBB…`,
/// `DEBE…`), in the registry of the process.
pub fn region_of_object_id(oid: &str) -> Option<AdapterId> {
    registry()
        .adapters()
        .find(|adapter| {
            adapter
                .info()
                .object_id_prefix
                .as_deref()
                .is_some_and(|prefix| oid.starts_with(prefix))
        })
        .map(|adapter| adapter.info().id.clone())
}

/// The ids of the built-in regions and providers, and what the tests ask of an id. Test and
/// `golden` feature only: the engine itself names no region.
#[cfg(any(test, feature = "golden"))]
pub mod ids {
    use freemapper_adapter_api::geometry::Outline as _;
    use freemapper_adapter_api::rasters::RasterKind;
    use geo_types::MultiPolygon;

    use super::{AdapterId, adapter};
    use crate::adapters::registry;
    use crate::crs::Crs;

    /// The regions, by the id of their adapter.
    pub mod regions {
        use super::AdapterId;

        pub const BRANDENBURG: AdapterId = AdapterId::built_in("brandenburg");
        pub const BERLIN: AdapterId = AdapterId::built_in("berlin");
        pub const SAXONY: AdapterId = AdapterId::built_in("saxony");
        pub const SAXONY_ANHALT: AdapterId = AdapterId::built_in("saxonyanhalt");
        pub const HAMBURG: AdapterId = AdapterId::built_in("hamburg");
        pub const HESSEN: AdapterId = AdapterId::built_in("hessen");
        pub const SCHLESWIG_HOLSTEIN: AdapterId = AdapterId::built_in("schleswigholstein");
        pub const MECKLENBURG_VORPOMMERN: AdapterId = AdapterId::built_in("mecklenburgvorpommern");
        pub const NORDRHEIN_WESTFALEN: AdapterId = AdapterId::built_in("nordrheinwestfalen");
        pub const NIEDERSACHSEN: AdapterId = AdapterId::built_in("niedersachsen");
        pub const RHEINLAND_PFALZ: AdapterId = AdapterId::built_in("rheinlandpfalz");
        pub const SAARLAND: AdapterId = AdapterId::built_in("saarland");
        pub const BADEN_WUERTTEMBERG: AdapterId = AdapterId::built_in("badenwuerttemberg");

        /// All regions, in the order of the registry.
        pub const ALL: [AdapterId; 13] = [
            BRANDENBURG,
            BERLIN,
            SAXONY,
            SAXONY_ANHALT,
            HAMBURG,
            HESSEN,
            SCHLESWIG_HOLSTEIN,
            MECKLENBURG_VORPOMMERN,
            NORDRHEIN_WESTFALEN,
            NIEDERSACHSEN,
            RHEINLAND_PFALZ,
            SAARLAND,
            BADEN_WUERTTEMBERG,
        ];

        /// The eleven regions that were code before they became adapters: the regions of the
        /// golden net, which proves that the move changed nothing. A state added since
        /// (Saarland, Baden-Württemberg) has no recording there.
        pub const LEGACY: [AdapterId; 11] = [
            BRANDENBURG,
            BERLIN,
            SAXONY,
            SAXONY_ANHALT,
            HAMBURG,
            HESSEN,
            SCHLESWIG_HOLSTEIN,
            MECKLENBURG_VORPOMMERN,
            NORDRHEIN_WESTFALEN,
            NIEDERSACHSEN,
            RHEINLAND_PFALZ,
        ];

        /// The region with this id, if it is one of the registry.
        pub fn from_id(id: &str) -> Option<AdapterId> {
            super::adapter(id).map(|adapter| adapter.info().id.clone())
        }
    }

    /// The providers of the rasters, by their id.
    pub mod providers {
        use super::AdapterId;

        pub const LGB: AdapterId = AdapterId::built_in("lgb");
        pub const GEO_SN: AdapterId = AdapterId::built_in("geosn");
        pub const LVERM_GEO: AdapterId = AdapterId::built_in("lvermgeo");
        pub const LGV: AdapterId = AdapterId::built_in("lgv");
        pub const HVBG: AdapterId = AdapterId::built_in("hvbg");
        pub const LVERM_GEO_SH: AdapterId = AdapterId::built_in("lvermgeosh");
        pub const LAIV_MV: AdapterId = AdapterId::built_in("laivmv");
        pub const GEOBASIS_NRW: AdapterId = AdapterId::built_in("geobasisnrw");
        pub const LGLN: AdapterId = AdapterId::built_in("lgln");
        pub const LVERM_GEO_RP: AdapterId = AdapterId::built_in("lvermgeorp");
        pub const LVGL: AdapterId = AdapterId::built_in("lvgl");
        pub const LGL_BW: AdapterId = AdapterId::built_in("lglbw");
    }

    /// What the registry says about the region or the provider with an id.
    pub trait IdFacts {
        /// The id as text.
        fn id(&self) -> &str;
        /// The provider of the rasters of this region.
        fn provider(&self) -> AdapterId;
        /// The CRS of a map whose centre lies in this region.
        fn crs(&self) -> Crs;
        /// What the ALKIS object ids of this region start with.
        fn object_id_prefix(&self) -> String;
        /// The boundary of this region.
        fn boundary(&self) -> &'static MultiPolygon<f64>;
        /// Whether this provider fetches its rasters per map area.
        fn per_area(&self) -> bool;
        /// The terrain model the contours of this provider come from.
        fn terrain_source(&self) -> String;
    }

    fn region(id: &AdapterId) -> &'static dyn crate::adapters::RegionAdapter {
        adapter(id.as_str()).unwrap_or_else(|| panic!("no region {id}"))
    }

    fn backend(id: &AdapterId) -> &'static dyn freemapper_adapter_api::rasters::RasterBackend {
        registry()
            .adapters()
            .filter(|adapter| adapter.info().provider == *id)
            .find_map(|adapter| adapter.rasters())
            .unwrap_or_else(|| panic!("no provider {id}"))
    }

    impl IdFacts for AdapterId {
        fn id(&self) -> &str {
            self.as_str()
        }

        fn provider(&self) -> AdapterId {
            region(self).info().provider.clone()
        }

        fn crs(&self) -> Crs {
            region(self).info().crs
        }

        fn object_id_prefix(&self) -> String {
            region(self)
                .info()
                .object_id_prefix
                .clone()
                .unwrap_or_default()
        }

        fn boundary(&self) -> &'static MultiPolygon<f64> {
            freemapper_adapters_de::boundaries::outline(self.as_str())
                .unwrap_or_else(|| panic!("no boundary of {self}"))
                .boundary()
        }

        fn per_area(&self) -> bool {
            backend(self).provider().kind == RasterKind::PerArea
        }

        fn terrain_source(&self) -> String {
            backend(self).provider().terrain_source.clone()
        }
    }
}

/// Every per-region fact the registry answers, for the golden net of the
/// adapter migration (`tests/adapters_golden.rs`, layer G1). Test and `golden` feature only.
#[cfg(any(test, feature = "golden"))]
pub mod golden {
    use sha2::{Digest as _, Sha256};

    use std::path::Path;

    use super::ids::{IdFacts as _, regions};
    use super::*;
    use crate::fetch::Cache;
    use crate::sources::osm::golden::{OsmFacts, extracts_of};

    /// The credit lines of one project record combination in one year.
    #[derive(Debug, Clone, PartialEq, serde::Serialize)]
    pub struct CreditFacts {
        pub year: i32,
        /// `alkis`: only ALKIS objects were fetched; `rasters`: rasters (and tiles) as well.
        pub records: &'static str,
        pub lines: Vec<String>,
    }

    /// What the registry says about one region.
    #[derive(Debug, Clone, PartialEq, serde::Serialize)]
    pub struct RegionFacts {
        pub id: String,
        pub provider: String,
        /// The provider's folder in the shared cache, as [`Cache`] names it; `None` for a
        /// provider that keeps nothing there. Four names are not the provider id, and a user's
        /// cache holds several GB under them.
        pub cache_dir: Option<String>,
        pub terrain_source: String,
        pub per_area: bool,
        pub crs_epsg: u32,
        pub object_id_prefix: String,
        pub boundary_sha256: String,
        pub osm: Vec<OsmFacts>,
        pub credit: Vec<CreditFacts>,
    }

    /// The object id prefix and a point (longitude, latitude) inside each region: the centre of
    /// its example area.
    fn constants(region: &AdapterId) -> (&'static str, f64, f64) {
        match region.as_str() {
            "brandenburg" => ("DEBB", 13.038, 52.403),
            "berlin" => ("DEBE", 13.381, 52.488),
            "saxony" => ("DESN", 14.421, 51.182),
            "saxonyanhalt" => ("DEST", 11.139, 51.788),
            "hamburg" => ("DEHH", 9.803, 53.5585),
            "hessen" => ("DEHE", 8.768, 50.81),
            "schleswigholstein" => ("DESH", 10.687, 53.866),
            "mecklenburgvorpommern" => ("DEMV", 11.465, 53.892),
            "nordrheinwestfalen" => ("DENW", 6.24, 50.555),
            "niedersachsen" => ("DENI", 9.6971, 52.388),
            "rheinlandpfalz" => ("DERP", 7.165, 50.1443),
            "saarland" => ("DESL", 6.7515, 49.3155),
            "badenwuerttemberg" => ("DEBW", 8.7125, 49.4112),
            other => panic!("no built-in region {other}"),
        }
    }

    /// A point (longitude, latitude) inside `region`: the centre of its example area and of its
    /// golden area.
    pub fn inside(region: impl AsRef<str>) -> (f64, f64) {
        let region: AdapterId = region.as_ref().parse().expect("a region id");
        let (_, lon, lat) = constants(&region);
        (lon, lat)
    }

    fn boundary_text(region: &AdapterId) -> &'static str {
        freemapper_adapters_de::boundaries::boundary_text(region.as_str())
            .expect("every built-in state has a boundary")
    }

    /// The folder `provider` keeps its downloads under in the shared cache: the name its
    /// module passes to `CacheDir::provider_dir`.
    fn cache_dir(provider: &AdapterId) -> Option<String> {
        let name = match provider.as_str() {
            "lgb" => "lgb",
            "geosn" => "geosn",
            "lgv" => "lgv",
            "hvbg" => "hvbg",
            "lvermgeosh" => "lvermgeo_sh",
            "geobasisnrw" => "geobasis_nrw",
            "lvermgeorp" => "lvermgeo_rp",
            "laivmv" => "laiv_mv",
            "lvgl" => "lvgl",
            "lglbw" => "lglbw",
            "lvermgeo" | "lgln" => return None,
            other => panic!("no built-in provider {other}"),
        };
        // The name is one `provider_dir` takes.
        let cache = Cache::new(Path::new("cache").to_path_buf());
        assert_eq!(cache.provider_dir(name), Path::new("cache").join(name));
        Some(name.to_string())
    }

    /// The facts of the eleven regions of [`regions::LEGACY`], in its order.
    pub fn legacy_facts() -> Vec<RegionFacts> {
        regions::LEGACY
            .into_iter()
            .map(|region| {
                let (prefix, lon, lat) = constants(&region);
                assert_eq!(region_of_object_id(prefix), Some(region.clone()));
                let provider = region.provider();
                let mut credit = Vec::new();
                for year in [2025, 2026] {
                    for (records, rasters) in [("alkis", false), ("rasters", true)] {
                        credit.push(CreditFacts {
                            year,
                            records,
                            lines: crate::project::golden::credit_lines(
                                &region, lon, lat, year, rasters,
                            ),
                        });
                    }
                }
                RegionFacts {
                    id: region.to_string(),
                    provider: provider.to_string(),
                    cache_dir: cache_dir(&provider),
                    terrain_source: provider.terrain_source(),
                    per_area: provider.per_area(),
                    crs_epsg: region.crs().epsg(),
                    object_id_prefix: prefix.to_string(),
                    boundary_sha256: Sha256::digest(boundary_text(&region).as_bytes())
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect(),
                    osm: extracts_of(&region),
                    credit,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn each_provider_names_its_own_terrain_source() {
        use super::ids::providers::*;
        let all = [
            LGB,
            GEO_SN,
            LVERM_GEO,
            LGV,
            HVBG,
            LVERM_GEO_SH,
            LAIV_MV,
            GEOBASIS_NRW,
            LGLN,
            LVERM_GEO_RP,
        ];
        let names: Vec<String> = all.iter().map(|p| p.terrain_source()).collect();
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

    use super::ids::{IdFacts as _, providers, regions};
    use super::*;

    /// A map area whose centre is in one state and whose edge is in another with another UTM
    /// zone: the CRS and the rasters follow the centre, ALKIS comes from both, in the order of
    /// the registry.
    #[test]
    fn border_centre_decides_crs() {
        let registry = registry();
        let (st, bb) = (
            registry.get("saxonyanhalt").unwrap(),
            registry.get("brandenburg").unwrap(),
        );
        // Zerbst lies in Saxony-Anhalt; going east, Brandenburg begins before Wiesenburg.
        let start = crate::crs::utm33_forward(crate::crs::LatLon {
            lat: 52.05,
            lon: 12.10,
        });
        let northing = start.northing.round();
        assert!(contains_point_in(st, start.easting, northing));
        let border = (0..800)
            .map(|step| start.easting.round() + f64::from(step) * 100.0)
            .find(|e| contains_point_in(bb, *e, northing))
            .expect("Brandenburg lies east of Zerbst");
        let area_round = |centre: f64| BoundingBox {
            min_e: centre - 400.0,
            min_n: northing - 200.0,
            max_e: centre + 400.0,
            max_n: northing + 200.0,
        };
        let ids = |area: &BoundingBox| -> Vec<&str> {
            regions_for_in(registry, area)
                .into_iter()
                .map(|a| a.info().id.as_str())
                .collect()
        };
        // Centre 300 m inside Saxony-Anhalt, the east edge in Brandenburg.
        let west = area_round(border - 300.0);
        let raster = raster_region_in(registry, &west).unwrap();
        assert_eq!(raster.info().id.as_str(), "saxonyanhalt");
        assert_eq!(raster.info().crs, crate::crs::Crs::Utm32);
        assert_eq!(ids(&west), ["brandenburg", "saxonyanhalt"]);
        // And the other way round.
        let east = area_round(border + 300.0);
        let raster = raster_region_in(registry, &east).unwrap();
        assert_eq!(raster.info().id.as_str(), "brandenburg");
        assert_eq!(raster.info().crs, crate::crs::Crs::Utm33);
        assert_eq!(ids(&east), ["brandenburg", "saxonyanhalt"]);
        // The functions that take ids say the same.
        assert_eq!(raster_region(&west), Some(regions::SAXONY_ANHALT));
        assert_eq!(
            regions_for(&west),
            [regions::BRANDENBURG, regions::SAXONY_ANHALT]
        );
    }

    /// The id is what the project file writes for the region; the readers name a state by it.
    #[test]
    fn the_id_of_a_region_is_its_name_in_the_project_file() {
        for region in regions::ALL {
            assert_eq!(serde_json::to_value(&region).unwrap(), region.id());
            assert_eq!(regions::from_id(region.id()), Some(region.clone()));
            assert_eq!(
                region_of_object_id(&format!("{}AL01", region.object_id_prefix())),
                Some(region.clone())
            );
            assert_eq!(
                serde_json::to_value(region.provider()).unwrap(),
                region.provider().id()
            );
        }
        assert_eq!(regions::from_id("bremen"), None);
    }

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
            vec![regions::BRANDENBURG]
        );
    }

    #[test]
    fn mitte_is_berlin() {
        assert_eq!(
            regions_for(&around(391_000.0, 5_820_000.0, 500.0)),
            vec![regions::BERLIN]
        );
    }

    #[test]
    fn border_needs_both() {
        // The southernmost boundary vertex, at Schönefeld.
        assert_eq!(
            regions_for(&around(407_850.0, 5_799_524.0, 300.0)),
            vec![regions::BRANDENBURG, regions::BERLIN]
        );
    }

    #[test]
    fn margin_reaches_across_the_border() {
        // 50 m south of the boundary: still within the margin.
        assert_eq!(
            regions_for(&around(407_850.0, 5_799_424.0, 50.0)),
            vec![regions::BRANDENBURG, regions::BERLIN]
        );
        // 300 m south: Brandenburg only.
        assert_eq!(
            regions_for(&around(407_850.0, 5_799_124.0, 50.0)),
            vec![regions::BRANDENBURG]
        );
    }

    #[test]
    fn dresden_is_saxony() {
        let school = around(415_973.0, 5_655_745.0, 300.0);
        assert_eq!(regions_for(&school), vec![regions::SAXONY]);
        assert_eq!(raster_region(&school), Some(regions::SAXONY));
        assert_eq!(regions::SAXONY.provider(), providers::GEO_SN);
    }

    #[test]
    fn lausitz_border_needs_both() {
        // A vertex shared by the Brandenburg and Saxony boundaries.
        assert_eq!(
            regions_for(&around(459_509.0, 5_710_017.0, 300.0)),
            vec![regions::BRANDENBURG, regions::SAXONY]
        );
    }

    #[test]
    fn goerlitz_east_edge_is_covered() {
        assert_eq!(
            raster_region(&around(501_000.0, 5_676_000.0, 300.0)),
            Some(regions::SAXONY)
        );
    }

    #[test]
    fn berlin_hole_is_not_brandenburg() {
        let mitte = around(391_000.0, 5_820_000.0, 500.0);
        assert!(!regions_for(&mitte).contains(&regions::BRANDENBURG));
        assert_eq!(raster_region(&mitte), Some(regions::BERLIN));
        assert_eq!(regions::BERLIN.provider(), providers::LGB);
        assert_eq!(regions::BRANDENBURG.provider(), providers::LGB);
    }

    #[test]
    fn raster_region_follows_the_centre() {
        // Centred 50 m south of Berlin's southernmost vertex: Brandenburg, although the box
        // reaches into Berlin.
        let south = around(407_850.0, 5_799_474.0, 200.0);
        assert!(regions_for(&south).contains(&regions::BERLIN));
        assert_eq!(raster_region(&south), Some(regions::BRANDENBURG));
        let north = around(407_850.0, 5_799_624.0, 200.0);
        assert_eq!(raster_region(&north), Some(regions::BERLIN));
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
                regions_for(&area).contains(&regions::NIEDERSACHSEN),
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
        assert_eq!(provider_tiles(&border, providers::LGB, 1), [at(410, 5691)]);
        assert_eq!(
            provider_tiles(&border, providers::GEO_SN, 2),
            [at(410, 5690)]
        );
    }

    #[test]
    fn halle_is_saxony_anhalt_in_zone_32() {
        // Latina August Hermann Francke, Halle (25833).
        let latina = around(289_795.0, 5_707_067.0, 200.0);
        assert_eq!(regions_for(&latina), vec![regions::SAXONY_ANHALT]);
        assert_eq!(raster_region(&latina), Some(regions::SAXONY_ANHALT));
        assert_eq!(regions::SAXONY_ANHALT.provider(), providers::LVERM_GEO);
        assert_eq!(regions::SAXONY_ANHALT.crs(), crate::crs::Crs::Utm32);
        assert_eq!(regions::SAXONY.crs(), crate::crs::Crs::Utm33);
        assert_eq!(
            region_of_object_id("DESTLIKA0001MuQqBL"),
            Some(regions::SAXONY_ANHALT)
        );
    }

    #[test]
    fn eppendorf_is_hamburg_in_zone_32() {
        // Gymnasium Eppendorf (25833).
        let school = around(168_078.0, 5_949_079.0, 200.0);
        assert_eq!(regions_for(&school), vec![regions::HAMBURG]);
        assert_eq!(raster_region(&school), Some(regions::HAMBURG));
        assert_eq!(regions::HAMBURG.provider(), providers::LGV);
        assert_eq!(regions::HAMBURG.crs(), crate::crs::Crs::Utm32);
        assert_eq!(
            region_of_object_id("DEHHALKAP0007dHTBL"),
            Some(regions::HAMBURG)
        );
    }

    #[test]
    fn frankfurt_school_is_hessen_in_zone_32() {
        // Lessing-Gymnasium Frankfurt (25833).
        let school = around(47_838.0, 5_571_740.0, 200.0);
        assert_eq!(regions_for(&school), vec![regions::HESSEN]);
        assert_eq!(raster_region(&school), Some(regions::HESSEN));
        assert_eq!(regions::HESSEN.provider(), providers::HVBG);
        assert!(providers::HVBG.per_area());
        assert_eq!(regions::HESSEN.crs(), crate::crs::Crs::Utm32);
        assert_eq!(crate::project::map_crs(&school), crate::crs::Crs::Utm32);
    }

    #[test]
    fn wiesbaden_with_negative_eastings_is_hessen() {
        // West of the 25833 central meridian's origin: eastings below zero.
        let west = around(-5_000.0, 5_564_000.0, 200.0);
        assert_eq!(regions_for(&west), vec![regions::HESSEN]);
        assert_eq!(raster_region(&west), Some(regions::HESSEN));
        assert!(no_buildings_warnings(&west).is_empty());
        // A per-area provider never has a 25833 tile grid.
        assert!(provider_tiles(&west, providers::HVBG, 1).is_empty());
        assert!(provider_tiles(&west, providers::HVBG, 2).is_empty());
    }

    #[test]
    fn hessen_object_ids() {
        assert_eq!(
            region_of_object_id("DEHE0000AbCdEfGh"),
            Some(regions::HESSEN)
        );
        assert_eq!(
            region_of_object_id("DEHH0000AbCdEfGh"),
            Some(regions::HAMBURG)
        );
    }

    #[test]
    fn norderstedt_school_is_schleswig_holstein_in_zone_32() {
        // Coppernicus-Gymnasium Norderstedt (OSM way/26986780), 25832 E 565214 / N 5949250.
        let school = around(169_108.0, 5_960_473.0, 200.0);
        assert_eq!(regions_for(&school), vec![regions::SCHLESWIG_HOLSTEIN]);
        assert_eq!(raster_region(&school), Some(regions::SCHLESWIG_HOLSTEIN));
        assert_eq!(
            regions::SCHLESWIG_HOLSTEIN.provider(),
            providers::LVERM_GEO_SH
        );
        assert!(providers::LVERM_GEO_SH.per_area());
        assert_eq!(regions::SCHLESWIG_HOLSTEIN.crs(), crate::crs::Crs::Utm32);
        assert_eq!(crate::project::map_crs(&school), crate::crs::Crs::Utm32);
        assert!(provider_tiles(&school, providers::LVERM_GEO_SH, 1).is_empty());
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
            vec![regions::HAMBURG, regions::SCHLESWIG_HOLSTEIN]
        );
        assert_eq!(raster_region(&north), Some(regions::SCHLESWIG_HOLSTEIN));
        // Centre in Hamburg (south of the border), box reaching into Schleswig-Holstein.
        let c = to_33(565_500.0, 5_947_100.0);
        let south = around(c.easting, c.northing, 700.0);
        assert_eq!(
            regions_for(&south),
            vec![regions::HAMBURG, regions::SCHLESWIG_HOLSTEIN]
        );
        assert_eq!(raster_region(&south), Some(regions::HAMBURG));
    }

    #[test]
    fn tile_touches_inside_on_the_border_and_outside() {
        let at = |east_km, north_km| TileId { east_km, north_km };
        // Hamburg-Langenhorn, well inside; Norderstedt, well inside Schleswig-Holstein.
        assert!(tile_touches(at(567, 5944), regions::HAMBURG));
        assert!(!tile_touches(at(567, 5944), regions::SCHLESWIG_HOLSTEIN));
        assert!(tile_touches(at(565, 5951), regions::SCHLESWIG_HOLSTEIN));
        assert!(!tile_touches(at(565, 5951), regions::HAMBURG));
        // On the border: 565/5948 is mostly Norderstedt but touches Hamburg; 566/5947 is mostly
        // Hamburg but 566/5948 reaches into Schleswig-Holstein.
        assert!(tile_touches(at(565, 5948), regions::HAMBURG));
        assert!(tile_touches(at(565, 5948), regions::SCHLESWIG_HOLSTEIN));
        assert!(tile_touches(at(566, 5948), regions::SCHLESWIG_HOLSTEIN));
        // Inside Hamburg just south of the border: 566/5947 does not reach Schleswig-Holstein.
        assert!(!tile_touches(at(566, 5947), regions::SCHLESWIG_HOLSTEIN));
        // Far from both states (E 400 km, N 5000 km): it touches neither.
        assert!(!tile_touches(at(400, 5000), regions::HAMBURG));
        assert!(!tile_touches(at(400, 5000), regions::SCHLESWIG_HOLSTEIN));
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
        assert_eq!(regions_for(&langenhorn), vec![regions::HAMBURG]);
        assert_eq!(raster_region(&langenhorn), Some(regions::HAMBURG));
    }

    #[test]
    fn schleswig_holstein_object_ids() {
        assert_eq!(
            region_of_object_id("DESH0000AbCdEfGh"),
            Some(regions::SCHLESWIG_HOLSTEIN)
        );
        assert_eq!(
            region_of_object_id("DEHH0000AbCdEfGh"),
            Some(regions::HAMBURG)
        );
    }

    #[test]
    fn rostock_school_is_mecklenburg_vorpommern_in_zone_33() {
        // Innerstaedtisches Gymnasium Rostock (OSM way/402355834), 25833 E 312157 / N 5996317.
        let school = around(312_157.0, 5_996_317.0, 200.0);
        assert_eq!(regions_for(&school), vec![regions::MECKLENBURG_VORPOMMERN]);
        assert_eq!(
            raster_region(&school),
            Some(regions::MECKLENBURG_VORPOMMERN)
        );
        assert_eq!(
            regions::MECKLENBURG_VORPOMMERN.provider(),
            providers::LAIV_MV
        );
        assert!(providers::LAIV_MV.per_area());
        assert_eq!(
            regions::MECKLENBURG_VORPOMMERN.crs(),
            crate::crs::Crs::Utm33
        );
        assert_eq!(crate::project::map_crs(&school), crate::crs::Crs::Utm33);
        assert!(provider_tiles(&school, providers::LAIV_MV, 1).is_empty());
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
        assert!(regions.contains(&regions::BRANDENBURG));
        assert!(regions.contains(&regions::MECKLENBURG_VORPOMMERN));
        // The raster region follows the centre; both sides stay in zone 33.
        assert_eq!(
            raster_region(&border),
            Some(regions::MECKLENBURG_VORPOMMERN)
        );
        assert_eq!(crate::project::map_crs(&border), crate::crs::Crs::Utm33);
        // A map centred well inside either side takes that side's provider.
        let north = around(437_007.0, 5_919_400.0, 100.0);
        let south = around(437_007.0, 5_917_900.0, 100.0);
        let providers: Vec<_> = [north, south]
            .iter()
            .map(|b| raster_region(b).unwrap().provider())
            .collect();
        assert!(providers.contains(&providers::LAIV_MV));
        assert!(providers.contains(&providers::LGB));
    }

    #[test]
    fn mv_object_ids() {
        assert_eq!(
            region_of_object_id("DEMV0000AbCdEfGh"),
            Some(regions::MECKLENBURG_VORPOMMERN)
        );
        assert_eq!(
            region_of_object_id("DEBB0000AbCdEfGh"),
            Some(regions::BRANDENBURG)
        );
    }

    #[test]
    fn koeln_school_is_nrw_in_zone_32() {
        // Hildegard-von-Bingen-Gymnasium Koeln-Suelz (OSM way/56752699), 25833 E -66680 /
        // N 5671681, 25832 E 354704 / N 5642657 (tile 32 354 5642).
        let school = around(-66_680.0, 5_671_681.0, 200.0);
        assert_eq!(regions_for(&school), vec![regions::NORDRHEIN_WESTFALEN]);
        assert_eq!(raster_region(&school), Some(regions::NORDRHEIN_WESTFALEN));
        assert_eq!(
            regions::NORDRHEIN_WESTFALEN.provider(),
            providers::GEOBASIS_NRW
        );
        assert!(providers::GEOBASIS_NRW.per_area());
        assert_eq!(regions::NORDRHEIN_WESTFALEN.crs(), crate::crs::Crs::Utm32);
        assert_eq!(crate::project::map_crs(&school), crate::crs::Crs::Utm32);
        assert!(provider_tiles(&school, providers::GEOBASIS_NRW, 1).is_empty());
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
            serde_json::to_string(&regions::NORDRHEIN_WESTFALEN).unwrap(),
            r#""nordrheinwestfalen""#
        );
        assert_eq!(
            serde_json::to_string(&providers::GEOBASIS_NRW).unwrap(),
            r#""geobasisnrw""#
        );
    }

    #[test]
    fn aachen_negative_eastings_is_nrw() {
        // Aachen cathedral (50.7753 N, 6.0839 E), 25833 E -128141 / N 5662838: far west of the
        // zone-33 origin.
        let aachen = around(-128_141.0, 5_662_838.0, 300.0);
        assert_eq!(regions_for(&aachen), vec![regions::NORDRHEIN_WESTFALEN]);
        assert_eq!(raster_region(&aachen), Some(regions::NORDRHEIN_WESTFALEN));
        assert!(contains_point(
            regions::NORDRHEIN_WESTFALEN,
            -128_141.0,
            5_662_838.0
        ));
        assert!(no_buildings_warnings(&aachen).is_empty());
        assert_eq!(crate::project::map_crs(&aachen), crate::crs::Crs::Utm32);
        assert!(provider_tiles(&aachen, providers::GEOBASIS_NRW, 1).is_empty());
        assert!(provider_tiles(&aachen, providers::GEOBASIS_NRW, 2).is_empty());
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
        assert_eq!(regions_for(&school), vec![regions::NIEDERSACHSEN]);
        assert_eq!(raster_region(&school), Some(regions::NIEDERSACHSEN));
        assert_eq!(regions::NIEDERSACHSEN.provider(), providers::LGLN);
        assert!(providers::LGLN.per_area());
        assert_eq!(regions::NIEDERSACHSEN.crs(), crate::crs::Crs::Utm32);
        assert_eq!(crate::project::map_crs(&school), crate::crs::Crs::Utm32);
        assert!(provider_tiles(&school, providers::LGLN, 1).is_empty());
        assert_eq!(
            serde_json::to_string(&regions::NIEDERSACHSEN).unwrap(),
            r#""niedersachsen""#
        );
        assert_eq!(
            serde_json::to_string(&providers::LGLN).unwrap(),
            r#""lgln""#
        );
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
                regions::NIEDERSACHSEN,
                p.easting,
                p.northing
            ));
        }
        // Verden (an der Aller), 30 km south-east of Bremen, is Niedersachsen.
        let verden = around_lonlat(52.9227, 9.2287, 300.0);
        assert_eq!(regions_for(&verden), vec![regions::NIEDERSACHSEN]);
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
            vec![regions::HAMBURG]
        );
        assert_eq!(
            regions_for(&around(seevetal.easting, seevetal.northing, 300.0)),
            vec![regions::NIEDERSACHSEN]
        );
        // Walk south from Harburg to the state border: the first box touching both regions has its
        // raster provider follow its centre.
        let mut found = false;
        for step in 0..120 {
            let n = harburg.northing - f64::from(step) * 50.0;
            let area = around(harburg.easting, n, 700.0);
            if regions_for(&area) == vec![regions::HAMBURG, regions::NIEDERSACHSEN] {
                found = true;
                let centre_region = if contains_point(regions::HAMBURG, harburg.easting, n) {
                    regions::HAMBURG
                } else {
                    regions::NIEDERSACHSEN
                };
                assert_eq!(raster_region(&area), Some(centre_region));
            }
        }
        assert!(found, "no box on the Hamburg / Niedersachsen border");
    }

    #[test]
    fn ni_object_ids() {
        assert_eq!(
            region_of_object_id("DENI0300AbCdEfGh"),
            Some(regions::NIEDERSACHSEN)
        );
        assert_eq!(
            region_of_object_id("DENW37AL0000AbCd"),
            Some(regions::NORDRHEIN_WESTFALEN)
        );
    }

    /// Saarland, the first state added as an adapter: Saarbrücken and Merzig lie in it, in
    /// zone 32, with rasters fetched per area by the LVGL; Zweibrücken next door is
    /// Rhineland-Palatinate.
    #[test]
    fn saarbruecken_lies_in_saarland_and_merzig_too() {
        for (lat, lon) in [(49.2337, 6.9941), (49.4430, 6.6370), (49.3155, 6.7515)] {
            let place = around_lonlat(lat, lon, 200.0);
            assert_eq!(regions_for(&place), vec![regions::SAARLAND], "{lat} {lon}");
            assert_eq!(raster_region(&place), Some(regions::SAARLAND));
            assert_eq!(crate::project::map_crs(&place), crate::crs::Crs::Utm32);
        }
        assert_eq!(regions::SAARLAND.provider(), providers::LVGL);
        assert!(providers::LVGL.per_area());
        assert_eq!(regions::SAARLAND.object_id_prefix(), "DESL");
        assert_eq!(
            region_of_object_id("DESLALKI4T001GlcBL"),
            Some(regions::SAARLAND)
        );
        let zweibruecken = around_lonlat(49.2490, 7.3630, 200.0);
        assert_eq!(regions_for(&zweibruecken), vec![regions::RHEINLAND_PFALZ]);
        // On the border both states are asked, as at every border.
        let homburg_edge = around_lonlat(49.3170, 7.3990, 400.0);
        let both = regions_for(&homburg_edge);
        assert!(both.contains(&regions::SAARLAND), "{both:?}");
    }

    /// Baden-Württemberg: Stuttgart, Freiburg (west of the zero meridian of zone 33, where
    /// eastings are negative) and Mannheim lie in it, in zone 32, with rasters fetched per area
    /// by the LGL. Across the Rhine Ludwigshafen is Rhineland-Palatinate, Viernheim is Hesse,
    /// and Neu-Ulm across the Danube lies in Bavaria, which no adapter covers.
    #[test]
    fn stuttgart_freiburg_and_mannheim_lie_in_baden_wuerttemberg() {
        for (lat, lon) in [(48.78741, 9.16135), (47.99046, 7.84076), (49.4870, 8.4660)] {
            let place = around_lonlat(lat, lon, 200.0);
            assert_eq!(
                regions_for(&place),
                vec![regions::BADEN_WUERTTEMBERG],
                "{lat} {lon}"
            );
            assert_eq!(raster_region(&place), Some(regions::BADEN_WUERTTEMBERG));
            assert_eq!(crate::project::map_crs(&place), crate::crs::Crs::Utm32);
        }
        assert!(around_lonlat(47.99046, 7.84076, 200.0).min_e < 0.0);
        assert_eq!(regions::BADEN_WUERTTEMBERG.provider(), providers::LGL_BW);
        assert!(providers::LGL_BW.per_area());
        assert_eq!(regions::BADEN_WUERTTEMBERG.object_id_prefix(), "DEBW");
        assert_eq!(
            region_of_object_id("DEBWL52210003w4m"),
            Some(regions::BADEN_WUERTTEMBERG)
        );
        let ludwigshafen = around_lonlat(49.4770, 8.4350, 200.0);
        assert_eq!(regions_for(&ludwigshafen), vec![regions::RHEINLAND_PFALZ]);
        let viernheim = around_lonlat(49.5400, 8.5790, 200.0);
        assert_eq!(regions_for(&viernheim), vec![regions::HESSEN]);
        let neu_ulm = around_lonlat(48.3890, 10.0200, 200.0);
        assert_eq!(regions_for(&neu_ulm), Vec::<AdapterId>::new());
        assert_eq!(raster_region(&neu_ulm), None);
        // On the Rhine both states are asked, as at every border.
        let harbour = around_lonlat(49.4870, 8.4480, 400.0);
        let both = regions_for(&harbour);
        assert!(both.contains(&regions::BADEN_WUERTTEMBERG), "{both:?}");
        assert!(both.contains(&regions::RHEINLAND_PFALZ), "{both:?}");
    }

    #[test]
    fn mainz_school_is_rp_in_zone_32() {
        // Frauenlob-Gymnasium Mainz-Neustadt (OSM way/60400704), 50.00908 N 8.26574 E.
        let school = around_lonlat(50.00908, 8.26574, 200.0);
        assert_eq!(regions_for(&school), vec![regions::RHEINLAND_PFALZ]);
        assert_eq!(raster_region(&school), Some(regions::RHEINLAND_PFALZ));
        assert_eq!(regions::RHEINLAND_PFALZ.provider(), providers::LVERM_GEO_RP);
        assert!(providers::LVERM_GEO_RP.per_area());
        assert_eq!(regions::RHEINLAND_PFALZ.crs(), crate::crs::Crs::Utm32);
        assert_eq!(crate::project::map_crs(&school), crate::crs::Crs::Utm32);
        assert!(provider_tiles(&school, providers::LVERM_GEO_RP, 1).is_empty());
        assert_eq!(
            serde_json::to_string(&regions::RHEINLAND_PFALZ).unwrap(),
            r#""rheinlandpfalz""#
        );
        assert_eq!(
            serde_json::to_string(&providers::LVERM_GEO_RP).unwrap(),
            r#""lvermgeorp""#
        );
        // Koblenz, Goerres-Gymnasium: negative 25833 eastings.
        let koblenz = around_lonlat(50.35964, 7.59917, 200.0);
        assert_eq!(regions_for(&koblenz), vec![regions::RHEINLAND_PFALZ]);
    }

    #[test]
    fn wiesbaden_stays_hessen() {
        // Wiesbaden Hauptbahnhof and Mainz-Kastel (Hesse, east of the Rhine).
        for (lat, lon) in [(50.0710, 8.2437), (50.0072, 8.2886)] {
            let area = around_lonlat(lat, lon, 150.0);
            assert_eq!(regions_for(&area), vec![regions::HESSEN], "{lat} {lon}");
            assert_eq!(raster_region(&area), Some(regions::HESSEN));
            let p = crate::crs::utm33_forward(crate::crs::LatLon { lat, lon });
            assert!(contains_point(regions::HESSEN, p.easting, p.northing));
            assert!(!contains_point(
                regions::RHEINLAND_PFALZ,
                p.easting,
                p.northing
            ));
        }
    }

    #[test]
    fn rp_object_ids() {
        assert_eq!(
            region_of_object_id("DERPLP050000AR8IBL"),
            Some(regions::RHEINLAND_PFALZ)
        );
        assert_eq!(
            region_of_object_id("DEHE0000AbCdEfGh"),
            Some(regions::HESSEN)
        );
    }

    #[test]
    fn nrw_object_ids() {
        assert_eq!(
            region_of_object_id("DENW37AL0000AbCd"),
            Some(regions::NORDRHEIN_WESTFALEN)
        );
        assert_eq!(
            region_of_object_id("DEHE0000AbCdEfGh"),
            Some(regions::HESSEN)
        );
    }

    #[test]
    fn object_ids_name_their_state() {
        assert_eq!(
            region_of_object_id("DEBE03YY600004AR"),
            Some(regions::BERLIN)
        );
        assert_eq!(
            region_of_object_id("DEBBAL0100000001"),
            Some(regions::BRANDENBURG)
        );
        assert_eq!(
            region_of_object_id("DESNALK0CrM0003UBL"),
            Some(regions::SAXONY)
        );
        assert_eq!(region_of_object_id("DETH01"), None);
    }

    #[test]
    fn touching_berlin_needs_no_margin() {
        // 50 m south of the boundary: within the fetch margin, but not in Berlin.
        assert!(no_buildings_warnings(&around(407_850.0, 5_799_424.0, 40.0)).is_empty());
        assert_eq!(
            no_buildings_warnings(&around(407_850.0, 5_799_524.0, 40.0)),
            ["warning-berlin-buildings-missing"]
        );
    }

    #[test]
    fn boundary_has_all_parts() {
        // After the validity repair (tests/region_assets.rs): an overlapping fragment merged into
        // the main polygon, slivers under 100 m² dropped.
        assert_eq!(regions::BERLIN.boundary().0.len(), 2);
        assert_eq!(regions::BERLIN.boundary().0[0].exterior().0.len(), 684);
        assert_eq!(
            regions::BRANDENBURG.boundary().0[0].interiors().len(),
            3,
            "Berlin and small enclaves"
        );
    }
}
