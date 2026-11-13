//! The OSM objects that tell who runs or protects an area.
//!
//! Only areas count: closed ways and multipolygon relations, decided by the geometry. The
//! selection is the list of the spec ("OSM selection"); nothing else is read. A private garden
//! and a historic house are left out, since their owner is often a person. Contact tags
//! (`email`, `phone`, `website`, `contact:*`) are never copied.

use crate::sources::osm::{OsmFeature, OsmGeometry};
use geo_types::{Coord, LineString, MultiPolygon, Polygon};
use serde::{Deserialize, Serialize};

/// Protected areas, in the order of the spec.
const PROTECTED: &[(&str, &str)] = &[
    ("boundary", "protected_area"),
    ("leisure", "nature_reserve"),
    ("boundary", "national_park"),
];

/// Public-kind objects whose `operator`/`owner` is wanted, in the order of the spec.
const OPERATED: &[(&str, &[&str])] = &[
    (
        "leisure",
        &["park", "garden", "sports_centre", "pitch", "stadium"],
    ),
    (
        "amenity",
        &[
            "school",
            "university",
            "college",
            "kindergarten",
            "place_of_worship",
            "hospital",
        ],
    ),
    (
        "landuse",
        &[
            "forest",
            "railway",
            "cemetery",
            "religious",
            "military",
            "education",
        ],
    ),
    ("natural", &["wood"]),
    ("tourism", &["attraction", "zoo", "theme_park"]),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OsmFindingKind {
    ProtectedArea,
    Operated,
}

/// One object of the selection. The geometry is not serialized here; the permit store writes
/// it as GeoJSON itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OsmFinding {
    pub kind: OsmFindingKind,
    pub osm_id: String,
    /// The first matching `key=value` of the selection, for example `leisure=park`.
    pub object: String,
    pub name: Option<String>,
    pub operator: Option<String>,
    pub owner: Option<String>,
    pub protect_class: Option<String>,
    pub protection_title: Option<String>,
    /// WGS 84, x = longitude, y = latitude.
    #[serde(skip, default = "empty_area")]
    pub geometry: MultiPolygon<f64>,
}

fn empty_area() -> MultiPolygon<f64> {
    MultiPolygon::new(vec![])
}

fn tag<'a>(tags: &'a [(String, String)], key: &str) -> Option<&'a str> {
    tags.iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.trim())
        .filter(|v| !v.is_empty())
}

/// `garden:type` values of a garden that belongs to a home.
const PRIVATE_GARDEN_TYPES: &[&str] = &["residential", "private"];

/// `access` values that close a garden to the public.
const CLOSED_ACCESS: &[&str] = &["private", "no"];

/// `building` values of a home: a `historic=*` object with one of them is a house.
const HOME_BUILDINGS: &[&str] = &[
    "house",
    "residential",
    "detached",
    "semidetached_house",
    "farm",
    "apartments",
];

/// A garden of a home or one closed to the public; its owner can be a person.
fn is_private_garden(tags: &[(String, String)]) -> bool {
    tag(tags, "garden:type").is_some_and(|v| PRIVATE_GARDEN_TYPES.contains(&v))
        || tag(tags, "access").is_some_and(|v| CLOSED_ACCESS.contains(&v))
}

/// The kind and `key=value` an object matches, or `None` when it is not in the selection.
fn classify(tags: &[(String, String)]) -> Option<(OsmFindingKind, String)> {
    for (key, value) in PROTECTED {
        if tag(tags, key) == Some(value) {
            return Some((OsmFindingKind::ProtectedArea, format!("{key}={value}")));
        }
    }
    if tag(tags, "operator").is_none() && tag(tags, "owner").is_none() {
        return None;
    }
    for (key, values) in OPERATED {
        if let Some(found) = tag(tags, key).filter(|v| values.contains(v)) {
            if *key == "leisure" && found == "garden" && is_private_garden(tags) {
                return None;
            }
            return Some((OsmFindingKind::Operated, format!("{key}={found}")));
        }
    }
    if tag(tags, "building").is_some_and(|v| HOME_BUILDINGS.contains(&v)) {
        return None;
    }
    tag(tags, "historic").map(|v| (OsmFindingKind::Operated, format!("historic={v}")))
}

/// Whether the tags belong to the OSM selection (protected area, or `operator`/`owner` on a
/// public-kind object). The geometry decides separately whether it is an area.
pub fn wants(tags: &[(String, String)]) -> bool {
    classify(tags).is_some()
}

fn ring(points: &[crate::crs::LatLon]) -> LineString<f64> {
    LineString::new(
        points
            .iter()
            .map(|p| Coord { x: p.lon, y: p.lat })
            .collect(),
    )
}

fn area(geometry: &OsmGeometry) -> Option<MultiPolygon<f64>> {
    match geometry {
        OsmGeometry::Point(_) => None,
        OsmGeometry::Line(points) => {
            let closed = points.len() >= 4 && points.first() == points.last();
            closed.then(|| MultiPolygon::new(vec![Polygon::new(ring(points), vec![])]))
        }
        OsmGeometry::Area(polygons) => {
            let polygons: Vec<_> = polygons
                .iter()
                .map(|p| Polygon::new(ring(&p.outer), p.holes.iter().map(|h| ring(h)).collect()))
                .collect();
            (!polygons.is_empty()).then(|| MultiPolygon::new(polygons))
        }
    }
}

/// The findings among `features`; copies only the keys of `OsmFinding`.
pub fn findings(features: Vec<OsmFeature>) -> Vec<OsmFinding> {
    features
        .into_iter()
        .filter_map(|feature| {
            let (kind, object) = classify(&feature.tags)?;
            let geometry = area(&feature.geometry)?;
            let text = |key: &str| tag(&feature.tags, key).map(str::to_string);
            Some(OsmFinding {
                kind,
                osm_id: feature.id.clone(),
                object,
                name: text("name"),
                operator: text("operator"),
                owner: text("owner"),
                protect_class: text("protect_class"),
                protection_title: text("protection_title"),
                geometry,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::LatLon;

    fn tags(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        let mut t: Vec<_> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        t.sort();
        t
    }

    fn square() -> OsmGeometry {
        let p = |lat, lon| LatLon { lat, lon };
        OsmGeometry::Line(vec![
            p(52.0, 13.0),
            p(52.0, 13.1),
            p(52.1, 13.1),
            p(52.1, 13.0),
            p(52.0, 13.0),
        ])
    }

    fn feature(id: &str, pairs: &[(&str, &str)], geometry: OsmGeometry) -> OsmFeature {
        OsmFeature {
            id: id.into(),
            tags: tags(pairs),
            geometry,
        }
    }

    #[test]
    fn a_park_with_an_operator_is_wanted() {
        assert!(wants(&tags(&[
            ("leisure", "park"),
            ("operator", "Stiftung Beispielgärten")
        ])));
        let found = findings(vec![feature(
            "way/1",
            &[("leisure", "park"), ("operator", "Stiftung Beispielgärten")],
            square(),
        )]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, OsmFindingKind::Operated);
        assert_eq!(found[0].object, "leisure=park");
        assert_eq!(
            found[0].operator.as_deref(),
            Some("Stiftung Beispielgärten")
        );
        assert_eq!(found[0].geometry.0.len(), 1);
    }

    #[test]
    fn historic_any_value_is_wanted() {
        assert!(wants(&tags(&[("historic", "ruins"), ("owner", "Land X")])));
    }

    /// A private garden can carry the name of a person as its owner (ADR 0117, anti-goals).
    #[test]
    fn a_private_garden_is_not_wanted() {
        for private in [
            ("garden:type", "residential"),
            ("garden:type", "private"),
            ("access", "private"),
            ("access", "no"),
        ] {
            let pairs = [("leisure", "garden"), ("owner", "Erika Beispiel"), private];
            assert!(!wants(&tags(&pairs)), "{private:?}");
            assert!(
                findings(vec![feature("way/7", &pairs, square())]).is_empty(),
                "{private:?}"
            );
        }
        // A public garden stays.
        assert!(wants(&tags(&[
            ("leisure", "garden"),
            ("garden:type", "botanical"),
            ("access", "yes"),
            ("operator", "Stiftung Beispielgärten")
        ])));
        // Access on another kind is not read: a school is often access=private.
        assert!(wants(&tags(&[
            ("amenity", "school"),
            ("access", "private"),
            ("operator", "Landkreis Beispiel")
        ])));
    }

    /// A listed house is `historic=*` too, and its owner is often a person.
    #[test]
    fn a_historic_house_is_not_wanted() {
        for building in [
            "house",
            "residential",
            "detached",
            "semidetached_house",
            "farm",
            "apartments",
        ] {
            let pairs = [
                ("historic", "building"),
                ("building", building),
                ("owner", "Erika Beispiel"),
            ];
            assert!(!wants(&tags(&pairs)), "{building}");
            assert!(
                findings(vec![feature("way/8", &pairs, square())]).is_empty(),
                "{building}"
            );
        }
        // A historic church or castle stays.
        assert!(wants(&tags(&[
            ("historic", "castle"),
            ("building", "castle"),
            ("owner", "Land X")
        ])));
    }

    #[test]
    fn residential_land_with_an_operator_is_not() {
        assert!(!wants(&tags(&[
            ("landuse", "residential"),
            ("operator", "Beispiel GmbH")
        ])));
    }

    #[test]
    fn a_house_with_an_owner_is_not() {
        assert!(!wants(&tags(&[
            ("building", "house"),
            ("owner", "Beispiel GmbH")
        ])));
    }

    #[test]
    fn a_park_without_operator_or_owner_is_not() {
        assert!(!wants(&tags(&[("leisure", "park")])));
    }

    #[test]
    fn a_protected_area_without_operator_is_wanted() {
        let found = findings(vec![feature(
            "relation/2",
            &[
                ("boundary", "protected_area"),
                ("name", "Beispielmoor"),
                ("protect_class", "4"),
            ],
            OsmGeometry::Area(vec![crate::sources::osm::OsmPolygon {
                outer: match square() {
                    OsmGeometry::Line(l) => l,
                    _ => unreachable!(),
                },
                holes: vec![],
            }]),
        )]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, OsmFindingKind::ProtectedArea);
        assert_eq!(found[0].protect_class.as_deref(), Some("4"));
        assert_eq!(found[0].operator, None);
    }

    #[test]
    fn contact_tags_never_reach_a_finding() {
        let found = findings(vec![feature(
            "way/3",
            &[
                ("leisure", "park"),
                ("operator", "Stiftung Beispielgärten"),
                ("email", "a@example.invalid"),
                ("phone", "+49 000 0000"),
                ("contact:email", "b@example.invalid"),
            ],
            square(),
        )]);
        let json = serde_json::to_string(&found).unwrap();
        for needle in ["email", "phone", "example.invalid", "+49", "contact"] {
            assert!(!json.contains(needle), "{needle} in {json}");
        }
    }

    #[test]
    fn a_node_is_not_an_area() {
        let found = findings(vec![feature(
            "node/4",
            &[("leisure", "park"), ("operator", "Stiftung Beispielgärten")],
            OsmGeometry::Point(LatLon {
                lat: 52.0,
                lon: 13.0,
            }),
        )]);
        assert!(found.is_empty());
    }

    #[test]
    fn an_open_way_is_not_an_area() {
        let mut open = match square() {
            OsmGeometry::Line(l) => l,
            _ => unreachable!(),
        };
        open.pop();
        let found = findings(vec![feature(
            "way/5",
            &[("boundary", "national_park")],
            OsmGeometry::Line(open),
        )]);
        assert!(found.is_empty());
    }
}
