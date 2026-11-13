//! Protected areas of the BfN (Bundesamt für Naturschutz): the nine kinds of its WFS
//! `schutzgebiet`, read by a box in WGS 84.
//!
//! The service answers GeoJSON in longitude, latitude (checked on the fixtures and live), so
//! the geometries come out as x = longitude, y = latitude without a swap.

use chrono::NaiveDate;
use geo_types::MultiPolygon;
use serde::{Deserialize, Serialize};
use std::path::Path;

use serde_json::Value;

use crate::fetch::{FetchError, Fetcher};
use crate::geojson_wfs::{GeoJsonWfs, LatLonBox, area_geometry, get_features, property_text};

/// The WFS of the BfN. It names its GeoJSON `GEOJSON` and reads a box in the CRS of the
/// `srsName`, so the box has no CRS of its own.
pub const BFN: GeoJsonWfs = GeoJsonWfs {
    base: "https://geodienste.bfn.de/ogc/wfs/schutzgebiet",
    output_format: "GEOJSON",
    bbox_crs_suffix: false,
};

/// The credit line of the BfN data; the caller fills `{year}` with the year of the fetch. The
/// BfN gives no wording of its own (GeoNutzV asks for a source note).
pub const BFN_CREDIT: &str = "© Bundesamt für Naturschutz (BfN) ({year}), Schutzgebiete in \
    Deutschland; Nutzungsbestimmungen für die Bereitstellung von Geodaten des Bundes (GeoNutzV). \
    Nicht für Planungszwecke geeignet.";

/// The access constraint of the service, verbatim.
pub const BFN_NOT_FOR_PLANNING: &str = "Nicht für Planungszwecke geeignet.";

/// The kinds of protected area the BfN serves, one feature type each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProtectedKind {
    Nationalpark,
    NationalesNaturmonument,
    Naturschutzgebiet,
    /// Fauna-Flora-Habitat-Gebiet (Natura 2000).
    Ffh,
    /// Vogelschutzgebiet (Natura 2000).
    Vogelschutzgebiet,
    Biosphaerenreservat,
    /// A zone of a Biosphärenreservat (Kern-, Pflege-, Entwicklungszone).
    BiosphaerenreservatZone,
    Landschaftsschutzgebiet,
    Naturpark,
}

impl ProtectedKind {
    /// Every kind, in the order the notes list them.
    pub const ALL: [ProtectedKind; 9] = [
        ProtectedKind::Nationalpark,
        ProtectedKind::NationalesNaturmonument,
        ProtectedKind::Naturschutzgebiet,
        ProtectedKind::Ffh,
        ProtectedKind::Vogelschutzgebiet,
        ProtectedKind::Biosphaerenreservat,
        ProtectedKind::BiosphaerenreservatZone,
        ProtectedKind::Landschaftsschutzgebiet,
        ProtectedKind::Naturpark,
    ];

    /// The feature type of the kind in the WFS.
    pub fn type_name(self) -> &'static str {
        match self {
            ProtectedKind::Nationalpark => "schutzgebiet:Nationalparke",
            ProtectedKind::NationalesNaturmonument => "schutzgebiet:Nationale_Naturmonumente",
            ProtectedKind::Naturschutzgebiet => "schutzgebiet:Naturschutzgebiete",
            ProtectedKind::Ffh => "schutzgebiet:Fauna_Flora_Habitat_Gebiete",
            ProtectedKind::Vogelschutzgebiet => "schutzgebiet:Vogelschutzgebiete",
            ProtectedKind::Biosphaerenreservat => "schutzgebiet:Biosphaerenreservate",
            ProtectedKind::BiosphaerenreservatZone => "schutzgebiet:Biosphaerenreservate_Zonierung",
            ProtectedKind::Landschaftsschutzgebiet => "schutzgebiet:Landschaftsschutzgebiete",
            ProtectedKind::Naturpark => "schutzgebiet:Naturparke",
        }
    }
}

/// One protected area as the BfN lists it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProtectedArea {
    pub kind: ProtectedKind,
    /// `NAME`; for a zone of a Biosphärenreservat followed by the zone in brackets. Empty when
    /// the BfN gives none.
    pub name: String,
    /// `LEG_DATE`, the date of the legal act.
    pub legal_date: Option<NaiveDate>,
    /// `LEG_TITEL`, the title of the legal act.
    pub legal_title: Option<String>,
    /// `LEG_LINK`, a link to the legal text.
    pub legal_link: Option<String>,
    /// `LINK`, the Natura 2000 site page of an FFH or bird area; no legal text.
    pub site_link: Option<String>,
    /// `JAHR`, the year the BfN gives the area.
    pub year: Option<i32>,
    /// The area in EPSG:4326, x = longitude, y = latitude.
    pub geometry: MultiPolygon<f64>,
}

/// The protected area of one feature of `kind`; `None` when it has no area geometry.
pub fn parse_bfn(kind: ProtectedKind, feature: &Value) -> Option<ProtectedArea> {
    let geometry = area_geometry(feature)?;
    let mut name = property_text(feature, "NAME").unwrap_or_default();
    if kind == ProtectedKind::BiosphaerenreservatZone
        && let Some(zone) = property_text(feature, "ZONIERUNG")
    {
        name = format!("{name} ({zone})");
    }
    Some(ProtectedArea {
        kind,
        name,
        legal_date: property_text(feature, "LEG_DATE").and_then(|d| legal_date(&d)),
        legal_title: property_text(feature, "LEG_TITEL"),
        legal_link: property_text(feature, "LEG_LINK"),
        // FFH and bird areas link their Natura 2000 page in `LINK`.
        site_link: property_text(feature, "LINK"),
        year: year(feature.get("properties").and_then(|p| p.get("JAHR"))),
        geometry,
    })
}

/// A `LEG_DATE` as the BfN writes it, month first (`8/1/1966`), or as an ISO date.
fn legal_date(text: &str) -> Option<NaiveDate> {
    let date = text.split_whitespace().next()?;
    NaiveDate::parse_from_str(date, "%m/%d/%Y")
        .or_else(|_| NaiveDate::parse_from_str(date, "%Y-%m-%d"))
        .ok()
}

/// A `JAHR` given as a number (`1966`, `2017.0`) or as text.
fn year(value: Option<&Value>) -> Option<i32> {
    let year = match value? {
        Value::Number(n) => n.as_f64()?,
        Value::String(s) => s.trim().parse::<f64>().ok()?,
        _ => return None,
    };
    (year.fract() == 0.0 && (1000.0..=9999.0).contains(&year)).then_some(year as i32)
}

/// The features of every kind inside `bbox`, one answer per kind in the order of
/// [`ProtectedKind::ALL`]. The features stay raw so the caller can store them; `part` is the
/// file each answer streams into.
pub fn fetch_bfn(
    fetcher: &dyn Fetcher,
    bbox: &LatLonBox,
    part: &Path,
) -> Vec<(ProtectedKind, Result<Vec<Value>, FetchError>)> {
    ProtectedKind::ALL
        .into_iter()
        .map(|kind| {
            (
                kind,
                get_features(fetcher, &BFN, kind.type_name(), bbox, part),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permits_test_support::{STRAUSBERG, centre, live_fetcher};
    use serde_json::json;

    const LSG: &str = include_str!("../tests/fixtures/permits/bfn-lsg-sanssouci.geojson");
    const NSG: &str = include_str!("../tests/fixtures/permits/bfn-nsg-strausberg.geojson");

    fn features(text: &str) -> Vec<Value> {
        let mut collection: Value = serde_json::from_str(text).unwrap();
        match collection["features"].take() {
            Value::Array(features) => features,
            _ => panic!("not a FeatureCollection"),
        }
    }

    #[test]
    fn bfn_lsg_fixture_parses() {
        let features = features(LSG);
        assert_eq!(features.len(), 1);
        let area = parse_bfn(ProtectedKind::Landschaftsschutzgebiet, &features[0]).unwrap();
        assert_eq!(area.kind, ProtectedKind::Landschaftsschutzgebiet);
        assert_eq!(area.name, "Potsdamer Havelseengebiet");
        assert_eq!(area.legal_date, NaiveDate::from_ymd_opt(1966, 8, 1));
        assert_eq!(
            area.legal_title.as_deref(),
            Some("Beschluß Nr. 149-14/66 des Rates des Bezirkes Potsdam vom 20.07.1966")
        );
        assert_eq!(area.legal_link, None);
        assert_eq!(area.year, Some(1966));
        // The BfN answers GeoJSON in longitude, latitude whatever the srsName says. The LSG
        // lies around Potsdam, its centre west of the town (12.96, 52.36 for the whole area).
        let (lon, lat) = centre(&area.geometry);
        assert!((12.9..13.1).contains(&lon), "lon {lon}");
        assert!((52.3..52.5).contains(&lat), "lat {lat}");
    }

    #[test]
    fn bfn_nsg_fixture_parses_in_longitude_latitude() {
        let areas: Vec<ProtectedArea> = features(NSG)
            .iter()
            .filter_map(|f| parse_bfn(ProtectedKind::Naturschutzgebiet, f))
            .collect();
        let names: Vec<&str> = areas.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "Herrensee, Lange-Damm-Wiesen und Barnim-Hänge",
                "Lange Dammwiesen und Unteres Annatal"
            ]
        );
        // The BfN writes a quote inside LEG_TITEL as `\"`; it comes out as a plain quote.
        let title = areas[0].legal_title.as_deref().unwrap();
        assert!(title.contains("zum LSG \"Strausberger Sander-"), "{title}");
        assert!(!title.contains('\\'), "{title}");
        assert_eq!(areas[1].legal_date, NaiveDate::from_ymd_opt(1990, 5, 16));
        for area in &areas {
            let (lon, lat) = centre(&area.geometry);
            assert!((13.8..13.95).contains(&lon), "lon {lon}");
            assert!((52.5..52.6).contains(&lat), "lat {lat}");
        }
    }

    #[test]
    fn bfn_blank_values_and_dates() {
        let feature = json!({
            "type": "Feature",
            "geometry": {"type": "Polygon", "coordinates": [[[13.0, 52.0], [13.1, 52.0], [13.1, 52.1], [13.0, 52.0]]]},
            "properties": {"NAME": "Ammergauer Alpen", "LEG_DATE": "8/1/1966", "LEG_LINK": " ", "LEG_TITEL": " ", "JAHR": 2017.0}
        });
        let area = parse_bfn(ProtectedKind::Naturpark, &feature).unwrap();
        assert_eq!(area.legal_date, NaiveDate::from_ymd_opt(1966, 8, 1));
        assert_eq!(area.legal_link, None);
        assert_eq!(area.legal_title, None);
        assert_eq!(area.year, Some(2017));
        assert_eq!(area.geometry.0.len(), 1);

        let blank = json!({
            "type": "Feature",
            "geometry": {"type": "Polygon", "coordinates": [[[13.0, 52.0], [13.1, 52.0], [13.1, 52.1], [13.0, 52.0]]]},
            "properties": {"NAME": "X", "LEG_DATE": " ", "LEG_LINK": null, "LEG_TITEL": "", "JAHR": " "}
        });
        let area = parse_bfn(ProtectedKind::Naturschutzgebiet, &blank).unwrap();
        assert_eq!(area.legal_date, None);
        assert_eq!(area.legal_title, None);
        assert_eq!(area.year, None);

        let year_as_text = json!({
            "type": "Feature",
            "geometry": {"type": "Polygon", "coordinates": [[[13.0, 52.0], [13.1, 52.0], [13.1, 52.1], [13.0, 52.0]]]},
            "properties": {"NAME": "X", "LEG_DATE": "2005-10-01", "JAHR": "2005"}
        });
        let area = parse_bfn(ProtectedKind::Naturschutzgebiet, &year_as_text).unwrap();
        assert_eq!(area.legal_date, NaiveDate::from_ymd_opt(2005, 10, 1));
        assert_eq!(area.year, Some(2005));
    }

    /// FFH and bird areas give their Natura 2000 site page in `LINK`; it is no legal text.
    #[test]
    fn the_natura2000_link_is_a_site_link() {
        let feature = |properties: Value| {
            json!({
                "type": "Feature",
                "geometry": {"type": "Polygon", "coordinates": [[[13.0, 52.0], [13.1, 52.0], [13.1, 52.1], [13.0, 52.0]]]},
                "properties": properties
            })
        };
        let ffh = parse_bfn(
            ProtectedKind::Ffh,
            &feature(json!({"NAME": "X", "LEG_LINK": " ", "LINK": "https://natura2000.example/DE3450301"})),
        )
        .unwrap();
        assert_eq!(ffh.legal_link, None);
        assert_eq!(
            ffh.site_link.as_deref(),
            Some("https://natura2000.example/DE3450301")
        );
        let both = parse_bfn(
            ProtectedKind::Vogelschutzgebiet,
            &feature(json!({"NAME": "X", "LEG_LINK": "https://example.org/vo", "LINK": "https://natura2000.example/DE3450401"})),
        )
        .unwrap();
        assert_eq!(both.legal_link.as_deref(), Some("https://example.org/vo"));
        assert_eq!(
            both.site_link.as_deref(),
            Some("https://natura2000.example/DE3450401")
        );
    }

    #[test]
    fn bfn_zone_goes_into_the_name() {
        let feature = json!({
            "type": "Feature",
            "geometry": {"type": "MultiPolygon", "coordinates": [[[[13.0, 52.0], [13.1, 52.0], [13.1, 52.1], [13.0, 52.0]]]]},
            "properties": {"NAME": "Schorfheide-Chorin", "ZONIERUNG": "Entwicklungszone"}
        });
        let area = parse_bfn(ProtectedKind::BiosphaerenreservatZone, &feature).unwrap();
        assert_eq!(area.name, "Schorfheide-Chorin (Entwicklungszone)");
    }

    #[test]
    fn bfn_feature_without_area_geometry_is_skipped() {
        for geometry in [
            Value::Null,
            json!({"type": "Point", "coordinates": [13.0, 52.0]}),
            json!({"type": "MultiPolygon"}),
        ] {
            let feature =
                json!({"type": "Feature", "geometry": geometry, "properties": {"NAME": "X"}});
            assert!(parse_bfn(ProtectedKind::Naturschutzgebiet, &feature).is_none());
        }
    }

    #[test]
    fn bfn_kinds_in_the_order_of_the_notes() {
        let names: Vec<&str> = ProtectedKind::ALL.iter().map(|k| k.type_name()).collect();
        assert_eq!(
            names,
            [
                "schutzgebiet:Nationalparke",
                "schutzgebiet:Nationale_Naturmonumente",
                "schutzgebiet:Naturschutzgebiete",
                "schutzgebiet:Fauna_Flora_Habitat_Gebiete",
                "schutzgebiet:Vogelschutzgebiete",
                "schutzgebiet:Biosphaerenreservate",
                "schutzgebiet:Biosphaerenreservate_Zonierung",
                "schutzgebiet:Landschaftsschutzgebiete",
                "schutzgebiet:Naturparke",
            ]
        );
        assert_eq!(
            serde_json::to_value(ProtectedKind::BiosphaerenreservatZone).unwrap(),
            json!("biosphaerenreservat-zone")
        );
        assert_eq!(
            serde_json::from_value::<ProtectedKind>(json!("ffh")).unwrap(),
            ProtectedKind::Ffh
        );
    }

    #[test]
    fn bfn_credit_has_a_year_to_fill() {
        assert!(BFN_CREDIT.contains("{year}"));
        assert!(BFN_CREDIT.ends_with(BFN_NOT_FOR_PLANNING));
        assert_eq!(BFN_NOT_FOR_PLANNING, "Nicht für Planungszwecke geeignet.");
    }

    #[test]
    fn fetch_bfn_asks_every_kind_with_geojson() {
        let fetcher = freemapper_adapter_api::testing::MapFetcher::default();
        let empty = br#"{"type":"FeatureCollection","features":[]}"#.to_vec();
        for kind in ProtectedKind::ALL {
            fetcher.insert(
                &crate::geojson_wfs::query_url(&BFN, kind.type_name(), &STRAUSBERG, 0),
                empty.clone(),
            );
        }
        let answers = fetch_bfn(
            &fetcher,
            &STRAUSBERG,
            &tempfile::tempdir().unwrap().path().join("page.part"),
        );
        assert_eq!(answers.len(), 9);
        for ((kind, answer), expected) in answers.iter().zip(ProtectedKind::ALL) {
            assert_eq!(*kind, expected);
            assert!(answer.as_ref().unwrap().is_empty());
        }
        assert!(
            crate::geojson_wfs::query_url(&BFN, "t", &STRAUSBERG, 0)
                .contains("outputFormat=GEOJSON")
        );
    }

    #[test]
    #[ignore = "asks the WFS of the BfN"]
    fn live_bfn_finds_the_nsg_of_strausberg() {
        let fetcher = live_fetcher();
        let policy = freemapper_adapter_api::policy::UrlPolicy::national();
        let fetcher = freemapper_adapter_api::policy::PolicyFetcher::new(&fetcher, &policy);
        let answers = fetch_bfn(
            &fetcher,
            &STRAUSBERG,
            &tempfile::tempdir().unwrap().path().join("page.part"),
        );
        let (_, nsg) = answers
            .iter()
            .find(|(kind, _)| *kind == ProtectedKind::Naturschutzgebiet)
            .unwrap();
        let areas: Vec<ProtectedArea> = nsg
            .as_ref()
            .unwrap()
            .iter()
            .filter_map(|f| parse_bfn(ProtectedKind::Naturschutzgebiet, f))
            .collect();
        assert!(areas.len() >= 2, "{} NSG", areas.len());
        for area in &areas {
            let (lon, lat) = centre(&area.geometry);
            assert!((13.7..14.0).contains(&lon) && (52.4..52.7).contains(&lat));
        }
    }
}
