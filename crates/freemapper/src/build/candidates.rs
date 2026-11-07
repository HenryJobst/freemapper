//! OSM features → candidate objects for the candidates template map (spec §4.5).

use chrono::NaiveDate;
use geo::{Distance as _, Euclidean};
use serde::Deserialize;

use crate::crs::{LatLon, utm33_forward};
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet, closed_ring};
use crate::project::MapKind;
use crate::sources::osm::{OsmFeature, OsmGeometry, Tags};

const EMBEDDED_RULES: &str = include_str!("../../../../assets/rules/osm-candidates.toml");

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("invalid candidate rules: {0}")]
pub struct RulesError(String);

/// How a matched feature is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Draw {
    Point,
    /// A node drawn as a horizontal line of this length on paper (mm).
    ShortLine(f64),
    Line,
    Area,
}

/// The symbol a feature becomes.
#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    pub code: String,
    pub draw: Draw,
    /// The tags the rule matched, `k=v; k=v`.
    pub matched: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DrawKind {
    Point,
    Line,
    Area,
}

#[derive(Debug, Deserialize)]
struct RuleFile {
    rule: Vec<RuleEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleEntry {
    when: Vec<String>,
    draw: DrawKind,
    point_as_short_line_mm: Option<f64>,
    schoolyard: Option<String>,
    sprint: Option<String>,
}

#[derive(Debug, Clone)]
struct Condition {
    key: String,
    /// `None` for `*`.
    values: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
struct Rule {
    when: Vec<Condition>,
    draw: DrawKind,
    point_as_short_line_mm: Option<f64>,
    schoolyard: Option<String>,
    sprint: Option<String>,
}

/// Candidate rules: OSM tags → symbol codes per map kind.
#[derive(Debug, Clone)]
pub struct Rules(Vec<Rule>);

fn value<'a>(tags: &'a Tags, key: &str) -> Option<&'a str> {
    tags.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

impl Condition {
    fn holds(&self, tags: &Tags) -> bool {
        match (value(tags, &self.key), &self.values) {
            (Some(_), None) => true,
            (Some(v), Some(values)) => values.iter().any(|allowed| allowed == v),
            (None, _) => false,
        }
    }
}

impl Rules {
    /// The rules shipped with freemapper (`assets/rules/osm-candidates.toml`).
    pub fn embedded() -> Self {
        Self::from_toml(EMBEDDED_RULES).expect("embedded candidate rules are valid")
    }

    pub fn from_toml(text: &str) -> Result<Self, RulesError> {
        let file: RuleFile = toml::from_str(text).map_err(|e| RulesError(e.to_string()))?;
        file.rule
            .into_iter()
            .map(|entry| {
                let when = entry
                    .when
                    .iter()
                    .map(|condition| {
                        let (key, values) = condition.split_once('=').ok_or_else(|| {
                            RulesError(format!("condition {condition:?} lacks `=`"))
                        })?;
                        Ok(Condition {
                            key: key.to_string(),
                            values: (values != "*")
                                .then(|| values.split('|').map(str::to_string).collect()),
                        })
                    })
                    .collect::<Result<Vec<_>, RulesError>>()?;
                Ok(Rule {
                    when,
                    draw: entry.draw,
                    point_as_short_line_mm: entry.point_as_short_line_mm,
                    schoolyard: entry.schoolyard,
                    sprint: entry.sprint,
                })
            })
            .collect::<Result<_, _>>()
            .map(Rules)
    }

    /// Whether any rule's conditions hold for `tags` (used to filter the extract).
    pub fn wants(&self, tags: &[(String, String)]) -> bool {
        let tags = tags.to_vec();
        self.0.iter().any(|r| r.when.iter().all(|c| c.holds(&tags)))
    }

    /// The symbol of the first rule whose conditions and geometry match; `None` when no rule
    /// matches or the matching rule has no symbol for `kind`.
    pub fn symbol_for(&self, tags: &Tags, geometry: &OsmGeometry, kind: MapKind) -> Option<Match> {
        let closed =
            matches!(geometry, OsmGeometry::Line(p) if p.len() > 3 && p.first() == p.last());
        let rule = self.0.iter().find_map(|rule| {
            if !rule.when.iter().all(|c| c.holds(tags)) {
                return None;
            }
            let draw = match (rule.draw, geometry) {
                (DrawKind::Point, OsmGeometry::Point(_)) => Draw::Point,
                (DrawKind::Line, OsmGeometry::Line(_)) => Draw::Line,
                (DrawKind::Line, OsmGeometry::Point(_)) => {
                    Draw::ShortLine(rule.point_as_short_line_mm?)
                }
                (DrawKind::Area, OsmGeometry::Line(_)) if closed => Draw::Area,
                _ => return None,
            };
            Some((rule, draw))
        })?;
        let (rule, draw) = rule;
        let code = match kind {
            MapKind::Schoolyard => rule.schoolyard.clone(),
            MapKind::Sprint => rule.sprint.clone(),
        }?;
        let matched = rule
            .when
            .iter()
            .filter_map(|c| Some(format!("{}={}", c.key, value(tags, &c.key)?)))
            .collect::<Vec<_>>()
            .join("; ");
        Some(Match {
            code,
            draw,
            matched,
        })
    }
}

/// Candidate objects for the features within `buffer_m` of `map_area` (EPSG:25833), tagged with
/// their provenance and the extract date as currency.
#[allow(clippy::too_many_arguments)]
pub fn candidate_objects(
    features: &[OsmFeature],
    rules: &Rules,
    kind: MapKind,
    map_area: &geo_types::Polygon<f64>,
    buffer_m: f64,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
    currency: NaiveDate,
) -> Result<Vec<MapObject>, GeorefError> {
    let projected = |p: &LatLon| {
        let q = utm33_forward(*p);
        geo_types::coord! { x: q.easting, y: q.northing }
    };
    let to_map = |p: &LatLon| -> Result<MapCoord, GeorefError> {
        georef.projected_to_map(utm33_forward(*p))
    };
    let mut objects = Vec::new();
    for feature in features {
        let Some(found) = rules.symbol_for(&feature.tags, &feature.geometry, kind) else {
            continue;
        };
        let Some(symbol_id) = symbol_set.id_for_code(&found.code) else {
            continue;
        };
        let distance = match &feature.geometry {
            OsmGeometry::Point(p) => Euclidean.distance(&geo_types::Point(projected(p)), map_area),
            OsmGeometry::Line(points) => Euclidean.distance(
                &geo_types::LineString(points.iter().map(projected).collect()),
                map_area,
            ),
        };
        if distance > buffer_m {
            continue;
        }
        let (kind, coords) = match (&feature.geometry, found.draw) {
            (OsmGeometry::Point(p), Draw::Point) => {
                (MapObjectKind::Point { rotation_rad: 0.0 }, vec![to_map(p)?])
            }
            (OsmGeometry::Point(p), Draw::ShortLine(mm)) => {
                let c = to_map(p)?;
                let half = (mm * 500.0).round() as i32;
                (
                    MapObjectKind::Path,
                    vec![
                        MapCoord::new(c.x - half, c.y),
                        MapCoord::new(c.x + half, c.y),
                    ],
                )
            }
            (OsmGeometry::Line(points), Draw::Line) => (
                MapObjectKind::Path,
                points.iter().map(to_map).collect::<Result<Vec<_>, _>>()?,
            ),
            (OsmGeometry::Line(points), Draw::Area) => {
                let ring = points
                    .iter()
                    .map(|p| to_map(p).map(|c| (c.x, c.y)))
                    .collect::<Result<Vec<_>, _>>()?;
                (MapObjectKind::Path, closed_ring(&ring))
            }
            _ => continue,
        };
        objects.push(MapObject {
            symbol_id,
            kind,
            coords,
            tags: vec![
                ("freemapper:source".to_string(), "osm".to_string()),
                ("freemapper:source_id".to_string(), feature.id.clone()),
                ("freemapper:osm".to_string(), found.matched),
                ("freemapper:currency".to_string(), currency.to_string()),
            ],
        });
    }
    Ok(objects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::LatLon;
    use crate::omap::coord::{FLAG_CLOSE, FLAG_HOLE};

    fn tags(pairs: &[(&str, &str)]) -> Tags {
        let mut tags: Tags = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        tags.sort();
        tags
    }

    const SCHOOL: LatLon = LatLon {
        lat: 52.54024,
        lon: 13.84434,
    };

    fn at(dlat: f64, dlon: f64) -> LatLon {
        LatLon {
            lat: SCHOOL.lat + dlat,
            lon: SCHOOL.lon + dlon,
        }
    }

    fn point(id: &str, t: &[(&str, &str)]) -> OsmFeature {
        OsmFeature {
            id: id.to_string(),
            tags: tags(t),
            geometry: OsmGeometry::Point(SCHOOL),
        }
    }

    fn line(id: &str, t: &[(&str, &str)], points: Vec<LatLon>) -> OsmFeature {
        OsmFeature {
            id: id.to_string(),
            tags: tags(t),
            geometry: OsmGeometry::Line(points),
        }
    }

    fn square() -> Vec<LatLon> {
        let d = 0.0002;
        vec![at(0.0, 0.0), at(0.0, d), at(d, d), at(d, 0.0), at(0.0, 0.0)]
    }

    struct Fixture {
        georef: Georeferencing,
        area: geo_types::Polygon<f64>,
    }

    fn fixture() -> Fixture {
        let date = "2026-11-07".parse().unwrap();
        let georef = Georeferencing::new(SCHOOL, 1000, date).unwrap();
        let (e, n) = (georef.ref_projected.easting, georef.ref_projected.northing);
        let ring = vec![
            (e - 60.0, n - 60.0),
            (e + 60.0, n - 60.0),
            (e + 60.0, n + 60.0),
            (e - 60.0, n + 60.0),
            (e - 60.0, n - 60.0),
        ];
        Fixture {
            georef,
            area: geo_types::Polygon::new(ring.into(), vec![]),
        }
    }

    fn objects(features: &[OsmFeature], kind: MapKind) -> (Vec<MapObject>, SymbolSet) {
        let f = fixture();
        let set = kind.symbol_set();
        let currency = "2026-11-06".parse().unwrap();
        let objects = candidate_objects(
            features,
            &Rules::embedded(),
            kind,
            &f.area,
            20.0,
            &f.georef,
            &set,
            currency,
        )
        .unwrap();
        (objects, set)
    }

    #[test]
    fn embedded_rule_codes_exist_in_the_symbol_sets() {
        for rule in &Rules::embedded().0 {
            for (code, kind) in [
                (&rule.schoolyard, MapKind::Schoolyard),
                (&rule.sprint, MapKind::Sprint),
            ] {
                if let Some(code) = code {
                    assert!(
                        kind.symbol_set().id_for_code(code).is_some(),
                        "{code} in {kind:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn rules_parse_and_first_match_wins() {
        let rules = Rules::from_toml(
            r#"
            [[rule]]
            when = ["amenity=bench"]
            draw = "point"
            schoolyard = "543"
            [[rule]]
            when = ["amenity=*"]
            draw = "point"
            schoolyard = "531"
            "#,
        )
        .unwrap();
        let bench = OsmGeometry::Point(SCHOOL);
        let found = rules.symbol_for(&tags(&[("amenity", "bench")]), &bench, MapKind::Schoolyard);
        assert_eq!(found.map(|m| m.code), Some("543".to_string()));
        let other = rules.symbol_for(
            &tags(&[("amenity", "toilets")]),
            &bench,
            MapKind::Schoolyard,
        );
        assert_eq!(other.map(|m| m.code), Some("531".to_string()));
        assert!(Rules::from_toml("[[rule]]\nwhen = [\"amenity\"]\ndraw = \"point\"").is_err());
        assert!(Rules::embedded().wants(&tags(&[("barrier", "fence")])));
        assert!(!Rules::embedded().wants(&tags(&[("building", "yes")])));
    }

    #[test]
    fn value_alternatives_and_wildcard() {
        let rules = Rules::embedded();
        let ring = OsmGeometry::Line(square());
        let asphalt = tags(&[("leisure", "track"), ("surface", "asphalt")]);
        let found = rules
            .symbol_for(&asphalt, &ring, MapKind::Schoolyard)
            .unwrap();
        assert_eq!((found.code.as_str(), found.draw), ("501.10", Draw::Area));
        let sand = tags(&[("leisure", "pitch"), ("surface", "sand")]);
        assert_eq!(rules.symbol_for(&sand, &ring, MapKind::Schoolyard), None);
        let slide = OsmGeometry::Point(SCHOOL);
        let found = rules
            .symbol_for(
                &tags(&[("playground", "slide")]),
                &slide,
                MapKind::Schoolyard,
            )
            .unwrap();
        assert_eq!(found.draw, Draw::ShortLine(1.8));
    }

    #[test]
    fn school_only_rules_skip_sprint() {
        let hydrant = point("node/1", &[("emergency", "fire_hydrant")]);
        assert!(objects(std::slice::from_ref(&hydrant), MapKind::Sprint).0.is_empty());
        assert_eq!(objects(&[hydrant], MapKind::Schoolyard).0.len(), 1);
    }

    #[test]
    fn hydrant_becomes_321_point() {
        let (objects, set) = objects(
            &[point("node/9669755236", &[("emergency", "fire_hydrant")])],
            MapKind::Schoolyard,
        );
        assert_eq!(Some(objects[0].symbol_id), set.id_for_code("321"));
        assert!(matches!(objects[0].kind, MapObjectKind::Point { .. }));
        assert_eq!(objects[0].coords.len(), 1);
    }

    #[test]
    fn bench_node_becomes_short_line() {
        let (objects, set) = objects(
            &[point("node/1", &[("amenity", "bench")])],
            MapKind::Schoolyard,
        );
        let bench = &objects[0];
        assert_eq!(Some(bench.symbol_id), set.id_for_code("543"));
        assert_eq!(bench.kind, MapObjectKind::Path);
        assert_eq!(bench.coords.len(), 2);
        assert_eq!(
            bench.coords[1].x - bench.coords[0].x,
            1500,
            "1.5 mm on paper"
        );
        assert_eq!(bench.coords[0].y, bench.coords[1].y);
    }

    #[test]
    fn fence_becomes_518_line() {
        let fence = line(
            "way/2",
            &[("barrier", "fence")],
            vec![at(0.0, 0.0), at(0.0001, 0.0002)],
        );
        let (objects, set) = objects(&[fence], MapKind::Sprint);
        assert_eq!(Some(objects[0].symbol_id), set.id_for_code("518"));
        assert_eq!(objects[0].coords.len(), 2);
        assert!(objects[0].coords.iter().all(|c| c.flags == 0));
    }

    #[test]
    fn tartan_pitch_becomes_501_10_area() {
        let pitch = line(
            "way/3",
            &[("leisure", "pitch"), ("surface", "tartan")],
            square(),
        );
        let (objects, set) = objects(&[pitch], MapKind::Schoolyard);
        assert_eq!(Some(objects[0].symbol_id), set.id_for_code("501.10"));
        assert_eq!(objects[0].coords.len(), 5);
        assert_eq!(objects[0].coords[4].flags, FLAG_HOLE | FLAG_CLOSE);
    }

    #[test]
    fn unclosed_area_way_is_skipped() {
        let mut open = square();
        open.pop();
        let pitch = line(
            "way/3",
            &[("leisure", "pitch"), ("surface", "tartan")],
            open,
        );
        assert!(objects(&[pitch], MapKind::Schoolyard).0.is_empty());
    }

    #[test]
    fn features_outside_buffer_are_skipped() {
        let mut far = point("node/4", &[("natural", "tree")]);
        // 60 m area half-width + 20 m buffer: 0.0015° of longitude are about 100 m.
        far.geometry = OsmGeometry::Point(at(0.0, 0.0015));
        let mut near = far.clone();
        near.geometry = OsmGeometry::Point(at(0.0, 0.0010));
        assert!(objects(&[far], MapKind::Sprint).0.is_empty());
        assert_eq!(objects(&[near], MapKind::Sprint).0.len(), 1);
    }

    #[test]
    fn candidate_tags_carry_provenance() {
        let pitch = line(
            "way/1028507308",
            &[
                ("leisure", "pitch"),
                ("sport", "multi"),
                ("surface", "tartan"),
            ],
            square(),
        );
        let (objects, _) = objects(&[pitch], MapKind::Schoolyard);
        let tags: Vec<(&str, &str)> = objects[0]
            .tags
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        assert_eq!(
            tags,
            [
                ("freemapper:source", "osm"),
                ("freemapper:source_id", "way/1028507308"),
                ("freemapper:osm", "leisure=pitch; surface=tartan"),
                ("freemapper:currency", "2026-11-06")
            ]
        );
    }
}
