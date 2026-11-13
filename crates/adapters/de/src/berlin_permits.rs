//! Berlin's sources of the permit notes (ADR 0120): five WFS of the Geoportal Berlin
//! (dl-de/zero-2-0) for the Bezirke, the green spaces and playgrounds, the land of the Land
//! Berlin and the Bund, the monuments and the protected objects under nature conservation law.
//!
//! All five are GeoServer WFS 2.0.0 that answer GeoJSON (`application/json`) and read a box in
//! WGS 84 latitude first when it ends with its CRS, which is how the generic reader asks
//! ([`get_features`]). The answers come as longitude, latitude; the parser still accepts
//! EPSG:25833 (the default CRS of the services) and latitude first, so a changed default does
//! not turn the findings into nonsense. A blank value of the services is a single space.
//!
//! The NSG, LSG, FFH, SPA and Naturpark layers of the `schutzgebiete` service are left out on
//! purpose: the BfN has them for every state, and the notes take each kind from one source.

use std::path::Path;

use freemapper_adapter_api::permits::{
    FindingKind, PermitFinding, PermitSection, PermitSource, PermitSourceInfo, ProtectedObjectKind,
    Wording,
};
use freemapper_sources::geojson_wfs::{GeoJsonWfs, LatLonBox, get_features};
use geo_types::{Coord, Geometry, Rect};
use serde_json::Value;

use crate::crs::{Crs, ProjectedPoint};
use crate::fetch::{FetchError, Fetcher};

/// The licence of all five services, verbatim from their capabilities (`Fees`).
pub const LICENCE: &str = "Für die Nutzung der Daten ist die Datenlizenz Deutschland - Zero - \
    Version 2.0 anzuwenden. Die Lizenz ist über https://www.govdata.de/dl-de/zero-2-0 abrufbar.";

/// What the protected objects are worth legally, verbatim from the abstract of the service
/// `schutzgebiete`.
pub const ANALOGUE_MAPS: &str = "Die amtlichen Karten zur jeweiligen Schutzverordnung können \
    bei der jeweils örtlich zuständigen unteren Behörde für Naturschutz und Landschaftspflege im \
    Bezirksamt oder bei der obersten Behörde für Naturschutz und Landschaftspflege \
    (Senatsverwaltung für Mobilität, Verkehr, Klimaschutz und Umwelt Abt. III) eingesehen \
    werden. Nur diese amtlichen analogen Karten sind rechtsverbindlich, sofern die Karten nicht \
    im Gesetz- und Verordnungsblatt für Berlin mitveröffentlicht wurden.";

const LABEL: Wording = Wording {
    en: "Geoportal Berlin",
    de: "Geoportal Berlin",
};

/// The owners the Liegenschaftsplan is named for. A parcel of any other owner counts as one of
/// another public owner, without its name: the notes never carry what the data might hold
/// beyond these two.
const NAMED_OWNERS: [&str; 2] = ["Land Berlin", "Bundesrepublik Deutschland"];

/// The link of a monument goes into this list; another link is not taken.
const MONUMENT_LIST: &str = "https://denkmaldatenbank.berlin.de/";

/// How the features of a layer become findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reading {
    District,
    GreenSpace,
    Playground,
    Owner,
    Monument,
    Protected(ProtectedObjectKind),
}

/// One service of the Geoportal Berlin with its layers: the key, the WFS type and how the
/// features are read, in the order of [`PermitSourceInfo::layers`].
pub struct BerlinPermitSource {
    info: PermitSourceInfo,
    service: GeoJsonWfs,
    layers: &'static [(&'static str, &'static str, Reading)],
}

const fn wfs(base: &'static str) -> GeoJsonWfs {
    GeoJsonWfs {
        base,
        output_format: "application/json",
        bbox_crs_suffix: true,
    }
}

const BEZIRKE: &[(&str, &str, Reading)] = &[(
    "bezirksgrenzen",
    "alkis_bezirke:bezirksgrenzen",
    Reading::District,
)];
const GRUENANLAGEN: &[(&str, &str, Reading)] = &[
    (
        "gruenanlagen",
        "gruenanlagen:gruenanlagen",
        Reading::GreenSpace,
    ),
    (
        "spielplaetze",
        "gruenanlagen:spielplaetze",
        Reading::Playground,
    ),
];
const LIEGENSCHAFTEN: &[(&str, &str, Reading)] = &[(
    "liegenschaftsplan",
    "liegenschaftsplan:liegenschaftsplan",
    Reading::Owner,
)];
const DENKMALE: &[(&str, &str, Reading)] = &[("denkmale", "denkmale:denkmale", Reading::Monument)];
const SCHUTZOBJEKTE: &[(&str, &str, Reading)] = &[
    (
        "a-nd",
        "schutzgebiete:a_nd",
        Reading::Protected(ProtectedObjectKind::NaturalMonument),
    ),
    (
        "e-fnd",
        "schutzgebiete:e_fnd",
        Reading::Protected(ProtectedObjectKind::NaturalMonument),
    ),
    (
        "g-glb",
        "schutzgebiete:g_glb",
        Reading::Protected(ProtectedObjectKind::ProtectedLandscapePart),
    ),
    (
        "f-fspr",
        "schutzgebiete:f_fspr",
        Reading::Protected(ProtectedObjectKind::SpecialRules),
    ),
];

/// One service of the five.
#[allow(clippy::too_many_arguments)]
fn source(
    key: &'static str,
    what: Wording,
    layers: &'static [&'static str],
    sections: &'static [PermitSection],
    note: Option<&'static str>,
    base: &'static str,
    table: &'static [(&'static str, &'static str, Reading)],
) -> Box<dyn PermitSource> {
    debug_assert!(
        layers.len() == table.len() && layers.iter().zip(table).all(|(a, (b, _, _))| a == b)
    );
    Box::new(BerlinPermitSource {
        info: PermitSourceInfo {
            key,
            label: LABEL,
            what,
            credit: LICENCE,
            note,
            layers,
            sections,
        },
        service: wfs(base),
        layers: table,
    })
}

/// The five services, in the order the notes list their sections.
pub fn sources() -> Vec<Box<dyn PermitSource>> {
    vec![
        source(
            "bezirke",
            Wording {
                en: "Bezirk borders (ALKIS Berlin)",
                de: "Bezirksgrenzen (ALKIS Berlin)",
            },
            &["bezirksgrenzen"],
            &[PermitSection::Districts],
            None,
            "https://gdi.berlin.de/services/wfs/alkis_bezirke",
            BEZIRKE,
        ),
        source(
            "gruenanlagen",
            Wording {
                en: "green spaces and public playgrounds",
                de: "Grünanlagenbestand mit den öffentlichen Spielplätzen",
            },
            &["gruenanlagen", "spielplaetze"],
            &[PermitSection::GreenSpaces],
            None,
            "https://gdi.berlin.de/services/wfs/gruenanlagen",
            GRUENANLAGEN,
        ),
        source(
            "liegenschaftsplan",
            Wording {
                en: "land of the Land Berlin and the Bund (Liegenschaftsplan)",
                de: "Liegenschaftsplan (Flurstücke des Bundeslandes Berlin und des Bundes)",
            },
            &["liegenschaftsplan"],
            &[PermitSection::Owners],
            None,
            "https://gdi.berlin.de/services/wfs/liegenschaftsplan",
            LIEGENSCHAFTEN,
        ),
        source(
            "denkmale",
            Wording {
                en: "monuments (Denkmalkarte)",
                de: "Denkmale (Denkmalkarte)",
            },
            &["denkmale"],
            &[PermitSection::Monuments],
            None,
            "https://gdi.berlin.de/services/wfs/denkmale",
            DENKMALE,
        ),
        source(
            "schutzgebiete",
            Wording {
                en: "protected objects under nature conservation law",
                de: "Schutzobjekte nach Naturschutzrecht",
            },
            &["a-nd", "e-fnd", "g-glb", "f-fspr"],
            &[PermitSection::ProtectedObjects],
            Some(ANALOGUE_MAPS),
            "https://gdi.berlin.de/services/wfs/schutzgebiete",
            SCHUTZOBJEKTE,
        ),
    ]
}

impl PermitSource for BerlinPermitSource {
    fn info(&self) -> &PermitSourceInfo {
        &self.info
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        bbox: &Rect<f64>,
        part: &Path,
    ) -> Result<Vec<Vec<Value>>, FetchError> {
        let bbox = LatLonBox {
            min_lat: bbox.min().y,
            min_lon: bbox.min().x,
            max_lat: bbox.max().y,
            max_lon: bbox.max().x,
        };
        self.layers
            .iter()
            .map(|(_, type_name, _)| get_features(fetcher, &self.service, type_name, &bbox, part))
            .collect()
    }

    fn findings(&self, layer: &str, features: &[Value]) -> Vec<PermitFinding> {
        let Some((_, _, reading)) = self.layers.iter().find(|(key, _, _)| *key == layer) else {
            return Vec::new();
        };
        features
            .iter()
            .filter_map(|feature| finding(*reading, feature))
            .collect()
    }
}

/// The finding of one feature; `None` without a usable geometry or without the attribute the
/// finding is about.
fn finding(reading: Reading, feature: &Value) -> Option<PermitFinding> {
    let geometry = geometry(feature)?;
    let area = matches!(geometry, Geometry::MultiPolygon(_));
    let kind = match reading {
        Reading::District => FindingKind::District {
            name: text(feature, "namgem")?,
        },
        Reading::GreenSpace => FindingKind::GreenSpace {
            name: text(feature, "namenr")
                .map(|name| park_name(&name))
                .unwrap_or_default(),
            district: text(feature, "bezirkname"),
            dedication: text(feature, "widmung"),
        },
        Reading::Playground => FindingKind::Playground,
        Reading::Owner => {
            let owner = text(feature, "sch")?;
            FindingKind::OwnerShare {
                owner: NAMED_OWNERS.contains(&owner.as_str()).then_some(owner),
            }
        }
        Reading::Monument => FindingKind::Monument {
            kind: text(feature, "typ")?,
            link: text(feature, "link").filter(|link| {
                link.starts_with(MONUMENT_LIST) && !link.contains(char::is_whitespace)
            }),
        },
        // The points of trees and boulders.
        Reading::Protected(kind) if !area => FindingKind::ProtectedObject {
            kind,
            name: text(feature, "baum_deutsch")
                .or_else(|| text(feature, "gestein_art"))
                .or_else(|| text(feature, "nd_art")),
            citation: text(feature, "zitat"),
        },
        Reading::Protected(kind) => FindingKind::ProtectedObject {
            kind,
            name: text(feature, "gebietsname"),
            citation: text(feature, "zitat"),
        },
    };
    // A district, a green space, a parcel is an area; only protected objects may be points.
    let point_allowed = matches!(reading, Reading::Protected(_) | Reading::Monument);
    (area || point_allowed).then_some(PermitFinding { kind, geometry })
}

/// The park a green space belongs to: Berlin names the parts of a park after it, with the part
/// after a comma ("Viktoriapark, Rosengarten"), and the notes give the park one line.
fn park_name(name: &str) -> String {
    name.split(", ").next().unwrap_or(name).trim().to_string()
}

/// The text of property `key`, trimmed; `None` when it is missing, not text or blank.
fn text(feature: &Value, key: &str) -> Option<String> {
    let text = feature.get("properties")?.get(key)?.as_str()?.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// The geometry of `feature` in WGS 84 with x = longitude: a MultiPolygon for an area, a
/// MultiPoint for an object.
fn geometry(feature: &Value) -> Option<Geometry<f64>> {
    let geometry =
        serde_json::from_value::<geojson::Geometry>(feature.get("geometry")?.clone()).ok()?;
    let geometry = match Geometry::<f64>::try_from(geometry).ok()? {
        Geometry::Polygon(p) => Geometry::MultiPolygon(geo_types::MultiPolygon(vec![p])),
        Geometry::Point(p) => Geometry::MultiPoint(geo_types::MultiPoint(vec![p])),
        g @ (Geometry::MultiPolygon(_) | Geometry::MultiPoint(_)) => g,
        _ => return None,
    };
    let geometry = lon_lat(geometry)?;
    let usable = match &geometry {
        Geometry::MultiPolygon(mp) => mp.0.iter().any(|p| p.exterior().0.len() >= 4),
        Geometry::MultiPoint(mp) => !mp.0.is_empty(),
        _ => false,
    };
    usable.then_some(geometry)
}

/// `geometry` as longitude, latitude. Coordinates in metres are EPSG:25833; degrees with the
/// first larger than the second are latitude first (in Berlin the latitude is about 52.5, the
/// longitude about 13.4). `None` when a position cannot be converted.
fn lon_lat(geometry: Geometry<f64>) -> Option<Geometry<f64>> {
    let projected = |c: &Coord<f64>| c.x.abs() > 360.0 || c.y.abs() > 360.0;
    let first = geo::CoordsIter::coords_iter(&geometry).next();
    let Some(first) = first else {
        return Some(geometry);
    };
    let convert: fn(Coord<f64>) -> Option<Coord<f64>> = if projected(&first) {
        |c| {
            let p = Crs::Utm33.inverse(ProjectedPoint {
                easting: c.x,
                northing: c.y,
            })?;
            Some(Coord { x: p.lon, y: p.lat })
        }
    } else if first.x > first.y {
        |c| Some(Coord { x: c.y, y: c.x })
    } else {
        return Some(geometry);
    };
    if geo::CoordsIter::coords_iter(&geometry).any(|c| convert(c).is_none()) {
        return None;
    }
    let mut geometry = geometry;
    geo::MapCoordsInPlace::map_coords_in_place(&mut geometry, |c| convert(c).unwrap_or(c));
    Some(geometry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::test_support::MapFetcher;

    macro_rules! fixture {
        ($name:literal) => {
            include_str!(concat!("../../../../tests/fixtures/permits-berlin/", $name))
        };
    }

    fn features(collection: &str) -> Vec<Value> {
        let value: Value = serde_json::from_str(collection).unwrap();
        value["features"].as_array().unwrap().clone()
    }

    fn source(key: &str) -> Box<dyn PermitSource> {
        sources().into_iter().find(|s| s.info().key == key).unwrap()
    }

    fn read(key: &str, layer: &str, collection: &str) -> Vec<PermitFinding> {
        source(key).findings(layer, &features(collection))
    }

    /// Every position of the findings lies around the Viktoriapark, as longitude, latitude.
    fn near_viktoriapark(findings: &[PermitFinding]) {
        for f in findings {
            for c in geo::CoordsIter::coords_iter(&f.geometry) {
                assert!((13.36..13.40).contains(&c.x), "{c:?}");
                assert!((52.48..52.50).contains(&c.y), "{c:?}");
            }
        }
    }

    #[test]
    fn five_sources_with_plain_keys() {
        let all = sources();
        let keys: Vec<&str> = all.iter().map(|s| s.info().key).collect();
        assert_eq!(
            keys,
            [
                "bezirke",
                "gruenanlagen",
                "liegenschaftsplan",
                "denkmale",
                "schutzgebiete"
            ]
        );
        for s in &all {
            let info = s.info();
            assert!(freemapper_adapter_api::permits::plain_key(info.key));
            assert!(
                info.layers
                    .iter()
                    .all(|l| freemapper_adapter_api::permits::plain_key(l))
            );
            assert_eq!(info.credit, LICENCE);
            assert_eq!(info.label, LABEL);
        }
        // Only the protected objects carry the note on the analogue maps.
        let notes: Vec<&str> = all.iter().filter_map(|s| s.info().note).collect();
        assert_eq!(notes, [ANALOGUE_MAPS]);
        assert!(
            ANALOGUE_MAPS.contains("Nur diese amtlichen analogen Karten sind rechtsverbindlich")
        );
    }

    #[test]
    fn the_queries_ask_the_geoportal_in_wgs84() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            "https://",
            br#"{"type":"FeatureCollection","features":[]}"#.to_vec(),
        );
        let part = tempfile::tempdir().unwrap();
        let bbox = Rect::new(
            Coord {
                x: 13.375,
                y: 52.486,
            },
            Coord {
                x: 13.389,
                y: 52.493,
            },
        );
        let answer = source("schutzgebiete")
            .fetch(&fetcher, &bbox, &part.path().join("x.part"))
            .unwrap();
        assert_eq!(answer.len(), 4);
        let log = fetcher.log.borrow();
        assert_eq!(log.len(), 4);
        for (url, type_name) in log.iter().zip(["a_nd", "e_fnd", "g_glb", "f_fspr"]) {
            assert!(
                url.starts_with("https://gdi.berlin.de/services/wfs/schutzgebiete?"),
                "{url}"
            );
            assert!(url.contains(&format!("typeNames=schutzgebiete:{type_name}&")));
            assert!(url.contains("outputFormat=application/json"), "{url}");
            assert!(
                url.ends_with("bbox=52.486,13.375,52.493,13.389,urn:ogc:def:crs:EPSG::4326"),
                "{url}"
            );
        }
    }

    #[test]
    fn a_failing_layer_stops_the_service() {
        let fetcher = MapFetcher::default();
        fetcher
            .refusing
            .borrow_mut()
            .push("https://gdi.berlin.de/".to_string());
        let part = tempfile::tempdir().unwrap();
        let bbox = Rect::new(Coord { x: 13.3, y: 52.4 }, Coord { x: 13.4, y: 52.5 });
        let source = source("gruenanlagen");
        assert!(
            source
                .fetch(&fetcher, &bbox, &part.path().join("x.part"))
                .is_err()
        );
        assert_eq!(fetcher.calls.get(), 1, "the playgrounds are not asked");
    }

    #[test]
    fn bezirke_are_named_by_namgem() {
        let found = read(
            "bezirke",
            "bezirksgrenzen",
            fixture!("bezirksgrenzen.geojson"),
        );
        let names: Vec<&FindingKind> = found.iter().map(|f| &f.kind).collect();
        assert_eq!(
            names,
            [
                &FindingKind::District {
                    name: "Friedrichshain-Kreuzberg".into()
                },
                &FindingKind::District {
                    name: "Tempelhof-Schöneberg".into()
                },
            ]
        );
        assert!(
            found
                .iter()
                .all(|f| matches!(f.geometry, Geometry::MultiPolygon(_)))
        );
        near_viktoriapark(&found);
    }

    #[test]
    fn green_spaces_take_the_park_name_and_drop_blanks() {
        let found = read(
            "gruenanlagen",
            "gruenanlagen",
            fixture!("gruenanlagen.geojson"),
        );
        assert_eq!(found.len(), 7);
        let names: Vec<String> = found
            .iter()
            .map(|f| match &f.kind {
                FindingKind::GreenSpace {
                    name,
                    district,
                    dedication,
                } => format!(
                    "{name} | {} | {}",
                    district.as_deref().unwrap_or("-"),
                    dedication.as_deref().unwrap_or("-")
                ),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(
            names[..2],
            [
                "Dora-Duncker-Park | Tempelhof-Schöneberg | gewidmet",
                "Viktoriapark | Friedrichshain-Kreuzberg | gewidmet",
            ]
        );
        assert!(names.contains(&"Mehringdamm 90 | Friedrichshain-Kreuzberg | gewidmet".into()));
        assert_eq!(
            names
                .iter()
                .filter(|n| n.starts_with("Viktoriapark |"))
                .count(),
            5,
            "{names:?}"
        );
        near_viktoriapark(&found);

        // A blank of a single space is no value.
        let blank = serde_json::json!({
            "type": "Feature",
            "geometry": {"type": "Polygon", "coordinates": [[[13.38, 52.49], [13.381, 52.49], [13.381, 52.491], [13.38, 52.49]]]},
            "properties": {"namenr": " ", "bezirkname": " ", "widmung": " "}
        });
        let found = source("gruenanlagen").findings("gruenanlagen", &[blank]);
        assert_eq!(
            found[0].kind,
            FindingKind::GreenSpace {
                name: String::new(),
                district: None,
                dedication: None
            }
        );
    }

    /// A park is the whole name before the first comma: parks whose names start with the same
    /// words stay apart, the parts of one park come together (review finding 5).
    #[test]
    fn parks_with_the_same_first_words_stay_apart() {
        let space = |name: &str| {
            serde_json::json!({
                "type": "Feature",
                "geometry": {"type": "Polygon", "coordinates": [[[13.38, 52.49], [13.381, 52.49], [13.381, 52.491], [13.38, 52.49]]]},
                "properties": {"namenr": name, "bezirkname": "Mitte", "widmung": "gewidmet"}
            })
        };
        let spaces: Vec<Value> = [
            "Park am Gleisdreieck, Ostpark",
            "Park am Gleisdreieck, Westpark",
            "Park am Weinberg, Spielwiese",
            "Volkspark Friedrichshain",
            "Volkspark Humboldthain, Nord",
        ]
        .map(space)
        .into();
        let names: Vec<String> = source("gruenanlagen")
            .findings("gruenanlagen", &spaces)
            .into_iter()
            .map(|f| match f.kind {
                FindingKind::GreenSpace { name, .. } => name,
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(
            names,
            [
                "Park am Gleisdreieck",
                "Park am Gleisdreieck",
                "Park am Weinberg",
                "Volkspark Friedrichshain",
                "Volkspark Humboldthain",
            ]
        );
    }

    #[test]
    fn playgrounds_are_findings_without_a_name() {
        let found = read(
            "gruenanlagen",
            "spielplaetze",
            fixture!("spielplaetze.geojson"),
        );
        assert_eq!(found.len(), 4);
        assert!(found.iter().all(|f| f.kind == FindingKind::Playground));
    }

    #[test]
    fn the_liegenschaftsplan_names_two_owners() {
        let found = read(
            "liegenschaftsplan",
            "liegenschaftsplan",
            fixture!("liegenschaftsplan.geojson"),
        );
        let owner = |name: &str| {
            found
                .iter()
                .filter(|f| {
                    f.kind
                        == FindingKind::OwnerShare {
                            owner: Some(name.into()),
                        }
                })
                .count()
        };
        assert_eq!(owner("Land Berlin"), 58);
        assert_eq!(owner("Bundesrepublik Deutschland"), 2);
        assert_eq!(found.len(), 60);
    }

    /// Only "Land Berlin" and "Bundesrepublik Deutschland" are named as owners; any other
    /// value counts as another public owner, without its text (review finding 8).
    #[test]
    fn only_the_land_and_the_bund_are_named_as_owners() {
        let parcel = |owner: &str| {
            serde_json::json!({
                "type": "Feature",
                "geometry": {"type": "Polygon", "coordinates": [[[13.38, 52.49], [13.381, 52.49], [13.381, 52.491], [13.38, 52.49]]]},
                "properties": {"sch": owner}
            })
        };
        let parcels: Vec<Value> = [
            " Land Berlin ",
            "Bundesrepublik Deutschland",
            "Muster Grundbesitz GmbH",
            "Land Berlin, Beispielverwaltung",
        ]
        .map(parcel)
        .into();
        let found = source("liegenschaftsplan").findings("liegenschaftsplan", &parcels);
        assert_eq!(found.len(), 4);
        let debug = format!("{found:?}");
        assert!(
            !debug.contains("Muster") && !debug.contains("Beispiel"),
            "{debug}"
        );
        let owners: Vec<Option<&str>> = found
            .iter()
            .map(|f| match &f.kind {
                FindingKind::OwnerShare { owner } => owner.as_deref(),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(
            owners,
            [
                Some("Land Berlin"),
                Some("Bundesrepublik Deutschland"),
                None,
                None
            ]
        );
    }

    #[test]
    fn monuments_carry_their_kind_and_link() {
        let found = read("denkmale", "denkmale", fixture!("denkmale.geojson"));
        assert_eq!(found.len(), 42);
        let garden: Vec<&FindingKind> = found
            .iter()
            .map(|f| &f.kind)
            .filter(|k| matches!(k, FindingKind::Monument { kind, .. } if kind == "Gartendenkmal"))
            .collect();
        assert_eq!(
            garden,
            [&FindingKind::Monument {
                kind: "Gartendenkmal".into(),
                link: Some(
                    "https://denkmaldatenbank.berlin.de/daobj.php?obj_dok_nr=09046157".into()
                ),
            }]
        );
        // A link into another site is not taken.
        let elsewhere = serde_json::json!({
            "type": "Feature",
            "geometry": {"type": "Point", "coordinates": [13.38, 52.49]},
            "properties": {"typ": "Baudenkmal", "link": "https://example.org/x"}
        });
        let found = source("denkmale").findings("denkmale", &[elsewhere]);
        assert_eq!(
            found[0].kind,
            FindingKind::Monument {
                kind: "Baudenkmal".into(),
                link: None
            }
        );
    }

    #[test]
    fn naturdenkmal_points_name_the_tree_and_cite_the_ordinance() {
        let found = read("schutzgebiete", "a-nd", fixture!("naturdenkmale.geojson"));
        assert_eq!(found.len(), 8);
        assert!(
            found
                .iter()
                .all(|f| matches!(f.geometry, Geometry::MultiPoint(_)))
        );
        assert_eq!(
            found[0].kind,
            FindingKind::ProtectedObject {
                kind: ProtectedObjectKind::NaturalMonument,
                name: Some("Ahornblättrige Platane".into()),
                citation: Some(
                    "Verordnung zum Schutz von Naturdenkmalen in Berlin vom 20.05.2021 (GVBl.S. 536)"
                        .into()
                ),
            }
        );
        near_viktoriapark(&found);
    }

    #[test]
    fn the_area_layers_read_gebietsname_and_zitat() {
        let area = |name: &str| {
            serde_json::json!({
                "type": "Feature",
                "geometry": {"type": "MultiPolygon", "coordinates": [[[[13.38, 52.49], [13.381, 52.49], [13.381, 52.491], [13.38, 52.49]]]]},
                "properties": {"gebietsname": name, "schutzstatus": "festgesetzt", "zitat": "VO vom 1.1.2000"}
            })
        };
        for (layer, kind) in [
            ("e-fnd", ProtectedObjectKind::NaturalMonument),
            ("g-glb", ProtectedObjectKind::ProtectedLandscapePart),
            ("f-fspr", ProtectedObjectKind::SpecialRules),
        ] {
            let found = source("schutzgebiete").findings(layer, &[area("Pfuhl")]);
            assert_eq!(
                found[0].kind,
                FindingKind::ProtectedObject {
                    kind,
                    name: Some("Pfuhl".into()),
                    citation: Some("VO vom 1.1.2000".into())
                },
                "{layer}"
            );
        }
        assert!(
            source("schutzgebiete")
                .findings("h-nsg", &[area("NSG")])
                .is_empty()
        );
    }

    /// The services' own CRS (EPSG:25833) and latitude first both come out as longitude,
    /// latitude.
    #[test]
    fn coordinates_are_normalised_to_lon_lat() {
        let lon_lat =
            features(fixture!("naturdenkmale.geojson"))[0]["geometry"]["coordinates"][0].clone();
        let (lon, lat) = (lon_lat[0].as_f64().unwrap(), lon_lat[1].as_f64().unwrap());
        let tree = |coordinates: Value| {
            serde_json::json!({
                "type": "Feature",
                "geometry": {"type": "MultiPoint", "coordinates": [coordinates]},
                "properties": {"baum_deutsch": "Platane"}
            })
        };
        // The point as the service answers in its default CRS.
        let projected = tree(serde_json::json!([390195.19597097, 5816323.24395383]));
        let swapped = tree(serde_json::json!([lat, lon]));
        for feature in [projected, swapped] {
            let found = source("schutzgebiete").findings("a-nd", &[feature]);
            let Geometry::MultiPoint(points) = &found[0].geometry else {
                panic!("{:?}", found[0].geometry);
            };
            assert!((points.0[0].x() - lon).abs() < 1e-6, "{points:?}");
            assert!((points.0[0].y() - lat).abs() < 1e-6, "{points:?}");
        }
    }

    #[test]
    fn a_district_needs_an_area() {
        let point = serde_json::json!({
            "type": "Feature",
            "geometry": {"type": "Point", "coordinates": [13.38, 52.49]},
            "properties": {"namgem": "Mitte"}
        });
        assert!(
            source("bezirke")
                .findings("bezirksgrenzen", &[point])
                .is_empty()
        );
    }

    /// The services for the Viktoriapark, asked live through the hosts of Berlin's manifest
    /// with the fixed User-Agent of the program:
    /// `cargo test -p freemapper-adapters-de live_viktoriapark_permit_sources -- --ignored`.
    #[test]
    #[ignore = "network"]
    fn live_viktoriapark_permit_sources() {
        use crate::fetch::HttpFetcher;
        use crate::states::{State, StateAdapter, register_backends};
        use freemapper_adapter_api::adapter::RegionAdapter as _;
        use freemapper_adapter_api::policy::PolicyFetcher;
        use freemapper_adapter_api::registry::Registry;
        assert!(!freemapper::fetch::USER_AGENT.contains('@'));
        let mut backends = Registry::new();
        register_backends(&mut backends).unwrap();
        let berlin = StateAdapter::new(State::Berlin, &backends);
        let http = HttpFetcher::new();
        let fetcher = PolicyFetcher::new(&http, berlin.policy().unwrap());
        // The probe box 389613,5816190,390529,5816909 (EPSG:25833) in WGS 84.
        let corner = |e, n| {
            let p = Crs::Utm33
                .inverse(ProjectedPoint {
                    easting: e,
                    northing: n,
                })
                .unwrap();
            Coord { x: p.lon, y: p.lat }
        };
        let (a, b) = (corner(389613.0, 5816190.0), corner(390529.0, 5816909.0));
        let bbox = Rect::new(a, b);
        let part = tempfile::tempdir().unwrap();
        let part = part.path().join("live.part");
        let mut found = std::collections::BTreeMap::new();
        for source in berlin.permit_sources() {
            let answers = source.fetch(&fetcher, &bbox, &part).unwrap();
            for (layer, features) in source.info().layers.iter().zip(&answers) {
                found.insert(*layer, source.findings(layer, features));
            }
        }
        let districts: std::collections::BTreeSet<String> = found["bezirksgrenzen"]
            .iter()
            .filter_map(|f| match &f.kind {
                FindingKind::District { name } => Some(name.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            districts.into_iter().collect::<Vec<_>>(),
            ["Friedrichshain-Kreuzberg", "Tempelhof-Schöneberg"]
        );
        let monuments = &found["denkmale"];
        let garden = monuments.iter().any(
            |f| matches!(&f.kind, FindingKind::Monument { kind, .. } if kind == "Gartendenkmal"),
        );
        assert!(garden || monuments.len() >= 30, "{}", monuments.len());
        let owners = |name: &str| {
            found["liegenschaftsplan"]
                .iter()
                .filter(|f| matches!(&f.kind, FindingKind::OwnerShare { owner } if owner.as_deref() == Some(name)))
                .count()
        };
        // The probe of 2026-11-13 found 58 and 2 parcels in the EPSG:25833 box; the box in
        // WGS 84 around its corners is a little larger and held 60 and 2 on the same day.
        assert!(owners("Land Berlin") >= 58, "{}", owners("Land Berlin"));
        assert_eq!(owners("Bundesrepublik Deutschland"), 2);
        // Whole Bezirke and parcels come back, so only the city is checked.
        for f in found.values().flatten() {
            for c in geo::CoordsIter::coords_iter(&f.geometry) {
                assert!(
                    (13.0..13.8).contains(&c.x) && (52.3..52.7).contains(&c.y),
                    "{c:?}"
                );
            }
        }
    }
}
