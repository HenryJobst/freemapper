//! Administrative units of the BKG (Verwaltungsgebiete 1:250 000, VG250): Land, Kreis,
//! Verwaltungsgemeinschaft and Gemeinde, read by a box in WGS 84.
//!
//! The GeoServer behind the service answers GeoJSON in longitude, latitude even though the
//! `srsName` is the EPSG URN (checked on the fixtures and live), so the geometries come out as
//! x = longitude, y = latitude without a swap. The units are generalised to 1:250 000.

use geo_types::MultiPolygon;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::fetch::{FetchError, Fetcher};
use crate::geojson_wfs::{GeoJsonWfs, LatLonBox, area_geometry, get_features, property_text};

/// The VG250 WFS of the BKG. GeoServer reads a box without a CRS in its own CRS, so the box
/// names EPSG:4326.
pub const VG250: GeoJsonWfs = GeoJsonWfs {
    base: "https://sgx.geodatenzentrum.de/wfs_vg250",
    output_format: "application/json",
    bbox_crs_suffix: true,
};

/// The credit line the service asks for, verbatim; the caller fills `{year}` with the year of
/// the fetch.
pub const BKG_CREDIT: &str = "© BKG ({year}) dl-de/by-2-0, Datenquellen: \
    https://sg.geodatenzentrum.de/web_public/gdz/datenquellen/datenquellen_vg_nuts.pdf";

/// The levels of administrative unit asked for, top down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdminLevel {
    Land,
    /// Landkreis or kreisfreie Stadt.
    Kreis,
    /// Amt, Verwaltungsgemeinschaft or amtsfreie Gemeinde.
    Verwaltungsgemeinschaft,
    Gemeinde,
}

impl AdminLevel {
    /// Every level, top down.
    pub const ALL: [AdminLevel; 4] = [
        AdminLevel::Land,
        AdminLevel::Kreis,
        AdminLevel::Verwaltungsgemeinschaft,
        AdminLevel::Gemeinde,
    ];

    /// The feature type of the level in the WFS.
    pub fn type_name(self) -> &'static str {
        match self {
            AdminLevel::Land => "vg250:vg250_lan",
            AdminLevel::Kreis => "vg250:vg250_krs",
            AdminLevel::Verwaltungsgemeinschaft => "vg250:vg250_vwg",
            AdminLevel::Gemeinde => "vg250:vg250_gem",
        }
    }
}

/// One administrative unit of VG250.
#[derive(Debug, Clone, PartialEq)]
pub struct AdminUnit {
    pub level: AdminLevel,
    /// `gen`, the name.
    pub name: String,
    /// `bez`, the kind of unit ("Stadt", "Landkreis", "Kreisfreie Stadt", "Amt", …).
    pub kind: String,
    /// `ags`, the official key.
    pub ags: String,
    /// The generalised area in EPSG:4326, x = longitude, y = latitude.
    pub geometry: MultiPolygon<f64>,
}

/// The unit of one feature of `level`; `None` when it has no area geometry.
pub fn parse_vg250(level: AdminLevel, feature: &Value) -> Option<AdminUnit> {
    Some(AdminUnit {
        level,
        geometry: area_geometry(feature)?,
        name: property_text(feature, "gen").unwrap_or_default(),
        kind: property_text(feature, "bez").unwrap_or_default(),
        ags: property_text(feature, "ags").unwrap_or_default(),
    })
}

/// The features of every level inside `bbox`, one answer per level in the order of
/// [`AdminLevel::ALL`]. The features stay raw so the caller can store them.
pub fn fetch_vg250(
    fetcher: &dyn Fetcher,
    bbox: &LatLonBox,
) -> Vec<(AdminLevel, Result<Vec<Value>, FetchError>)> {
    AdminLevel::ALL
        .into_iter()
        .map(|level| {
            (
                level,
                get_features(fetcher, &VG250, level.type_name(), bbox),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permits_test_support::{STRAUSBERG, centre, live_fetcher};
    use serde_json::json;

    const GEM: &str = include_str!("../tests/fixtures/permits/vg250-gem-sanssouci.geojson");
    const KRS: &str = include_str!("../tests/fixtures/permits/vg250-krs-sanssouci.geojson");

    fn features(text: &str) -> Vec<Value> {
        let mut collection: Value = serde_json::from_str(text).unwrap();
        match collection["features"].take() {
            Value::Array(features) => features,
            _ => panic!("not a FeatureCollection"),
        }
    }

    #[test]
    fn vg250_gemeinde_fixture_parses() {
        let features = features(GEM);
        assert_eq!(features.len(), 1);
        let unit = parse_vg250(AdminLevel::Gemeinde, &features[0]).unwrap();
        assert_eq!(unit.level, AdminLevel::Gemeinde);
        assert_eq!(unit.name, "Potsdam");
        assert_eq!(unit.kind, "Stadt");
        assert_eq!(unit.ags, "12054000");
        // GeoServer answers GeoJSON in longitude, latitude although the srsName is the EPSG URN.
        let (lon, lat) = centre(&unit.geometry);
        assert!((13.0..13.2).contains(&lon), "lon {lon}");
        assert!((52.3..52.5).contains(&lat), "lat {lat}");
    }

    #[test]
    fn vg250_kreis_fixture_parses() {
        let units: Vec<AdminUnit> = features(KRS)
            .iter()
            .filter_map(|f| parse_vg250(AdminLevel::Kreis, f))
            .collect();
        let found: Vec<(&str, &str, &str)> = units
            .iter()
            .map(|u| (u.name.as_str(), u.kind.as_str(), u.ags.as_str()))
            .collect();
        assert_eq!(
            found,
            [
                ("Potsdam", "Kreisfreie Stadt", "12054"),
                ("Potsdam-Mittelmark", "Landkreis", "12069"),
                ("Teltow-Fläming", "Landkreis", "12072"),
            ]
        );
        for unit in &units {
            let (lon, lat) = centre(&unit.geometry);
            assert!((12.5..13.5).contains(&lon), "lon {lon}");
            assert!((51.9..52.6).contains(&lat), "lat {lat}");
        }
    }

    #[test]
    fn vg250_feature_without_area_geometry_is_skipped() {
        let feature = json!({"type": "Feature", "geometry": null, "properties": {"gen": "X", "bez": "Stadt", "ags": "1"}});
        assert!(parse_vg250(AdminLevel::Gemeinde, &feature).is_none());
    }

    #[test]
    fn vg250_levels_top_down() {
        let names: Vec<&str> = AdminLevel::ALL.iter().map(|l| l.type_name()).collect();
        assert_eq!(
            names,
            [
                "vg250:vg250_lan",
                "vg250:vg250_krs",
                "vg250:vg250_vwg",
                "vg250:vg250_gem"
            ]
        );
        assert_eq!(
            serde_json::to_value(AdminLevel::Verwaltungsgemeinschaft).unwrap(),
            json!("verwaltungsgemeinschaft")
        );
    }

    #[test]
    fn bkg_credit_has_a_year_to_fill() {
        assert_eq!(
            BKG_CREDIT.replace("{year}", "2026"),
            "© BKG (2026) dl-de/by-2-0, Datenquellen: \
             https://sg.geodatenzentrum.de/web_public/gdz/datenquellen/datenquellen_vg_nuts.pdf"
        );
    }

    #[test]
    fn fetch_vg250_asks_every_level_with_the_crs_in_the_box() {
        let fetcher = freemapper_adapter_api::testing::MapFetcher::default();
        let empty = br#"{"type":"FeatureCollection","features":[]}"#.to_vec();
        for level in AdminLevel::ALL {
            let url = crate::geojson_wfs::query_url(&VG250, level.type_name(), &STRAUSBERG, 0);
            assert!(url.contains("outputFormat=application/json"));
            assert!(url.ends_with(",urn:ogc:def:crs:EPSG::4326"));
            fetcher.insert(&url, empty.clone());
        }
        let answers = fetch_vg250(&fetcher, &STRAUSBERG);
        let levels: Vec<AdminLevel> = answers.iter().map(|(l, _)| *l).collect();
        assert_eq!(levels, AdminLevel::ALL);
        assert!(answers.iter().all(|(_, a)| a.as_ref().unwrap().is_empty()));
    }

    #[test]
    #[ignore = "asks the WFS of the BKG"]
    fn live_vg250_finds_strausberg() {
        let fetcher = live_fetcher();
        let policy = freemapper_adapter_api::policy::UrlPolicy::national();
        let fetcher = freemapper_adapter_api::policy::PolicyFetcher::new(&fetcher, &policy);
        let answers = fetch_vg250(&fetcher, &STRAUSBERG);
        let (_, gem) = answers
            .iter()
            .find(|(level, _)| *level == AdminLevel::Gemeinde)
            .unwrap();
        let units: Vec<AdminUnit> = gem
            .as_ref()
            .unwrap()
            .iter()
            .filter_map(|f| parse_vg250(AdminLevel::Gemeinde, f))
            .collect();
        let strausberg = units.iter().find(|u| u.name == "Strausberg").unwrap();
        assert_eq!(strausberg.ags, "12064472");
        let (lon, lat) = centre(&strausberg.geometry);
        assert!((13.7..14.1).contains(&lon) && (52.4..52.7).contains(&lat));
    }
}
