//! OSM features → candidate objects for the candidates template map (spec §4.5).

use chrono::NaiveDate;
use geo::{BooleanOps as _, BoundingRect as _, Distance as _, Euclidean, Length as _};
use serde::Deserialize;

use crate::crs::{LatLon, ProjectedPoint};
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet, closed_ring};
use crate::project::MapKind;
use crate::sources::osm::{OsmFeature, OsmGeometry, Tags};

const EMBEDDED_RULES: &str = include_str!("../../../../assets/rules/osm-candidates.toml");
const BERLIN_RULES: &str = include_str!("../../../../assets/rules/berlin-candidates.toml");
/// Line pieces shorter than this on paper are left over from clipping at the buffer edge.
const MIN_LINE_MM: f64 = 1.0;

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

/// Where a matched feature goes: the candidates template map or the map itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    #[default]
    Candidates,
    Map,
}

/// The symbol a feature becomes.
#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    pub code: String,
    pub draw: Draw,
    pub target: Target,
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
    #[serde(default)]
    target: Target,
    schoolyard: Option<String>,
    sprint: Option<String>,
}

#[derive(Debug, Clone)]
enum Test {
    /// `key=a|b`; `None` for `*`.
    Values(Option<Vec<String>>),
    /// `key<number`: the value is a number below this.
    Below(f64),
}

#[derive(Debug, Clone)]
struct Condition {
    key: String,
    test: Test,
    /// `key!=value`: holds when the tag is absent or has none of the values.
    negated: bool,
}

#[derive(Debug, Clone)]
struct Rule {
    when: Vec<Condition>,
    draw: DrawKind,
    point_as_short_line_mm: Option<f64>,
    target: Target,
    schoolyard: Option<String>,
    sprint: Option<String>,
}

/// Candidate rules: OSM tags → symbol codes per map kind.
#[derive(Debug, Clone)]
pub struct Rules(Vec<Rule>);

fn value<'a>(tags: &'a Tags, key: &str) -> Option<&'a str> {
    tags.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

/// A tag value as a positive number: `1.2`, `1,2`, `1.2 m`.
fn number(value: &str) -> Option<f64> {
    let value = value.trim();
    let value = value.strip_suffix('m').unwrap_or(value).trim_end();
    let n: f64 = value.replace(',', ".").parse().ok()?;
    (n.is_finite() && n > 0.0).then_some(n)
}

impl Condition {
    fn holds(&self, tags: &Tags) -> bool {
        let matches = match (value(tags, &self.key), &self.test) {
            (Some(_), Test::Values(None)) => true,
            (Some(v), Test::Values(Some(values))) => values.iter().any(|allowed| {
                // `Mauerkante*` is a prefix (only the Berlin rules use it).
                allowed == v || allowed.strip_suffix('*').is_some_and(|p| v.starts_with(p))
            }),
            (Some(v), Test::Below(limit)) => number(v).is_some_and(|n| n < *limit),
            (None, _) => false,
        };
        matches != self.negated
    }
}

impl Rules {
    /// The rules shipped with freemapper (`assets/rules/osm-candidates.toml`).
    pub fn embedded() -> Self {
        Self::from_toml(EMBEDDED_RULES).expect("embedded candidate rules are valid")
    }

    /// The Geoportal Berlin rules (`assets/rules/berlin-candidates.toml`): `berlin:*` tags only.
    pub fn berlin() -> Self {
        Self::from_toml(BERLIN_RULES).expect("embedded Berlin rules are valid")
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
                        if !condition.contains('=')
                            && let Some((key, limit)) = condition.split_once('<')
                        {
                            let limit = number(limit).ok_or_else(|| {
                                RulesError(format!(
                                    "condition {condition:?}: {limit:?} is no number"
                                ))
                            })?;
                            return Ok(Condition {
                                key: key.to_string(),
                                test: Test::Below(limit),
                                negated: false,
                            });
                        }
                        let (key, values) = condition.split_once('=').ok_or_else(|| {
                            RulesError(format!("condition {condition:?} lacks `=` or `<`"))
                        })?;
                        let (key, negated) = match key.strip_suffix('!') {
                            Some(key) => (key, true),
                            None => (key, false),
                        };
                        if key.contains(['<', '>']) {
                            return Err(RulesError(format!(
                                "condition {condition:?}: only `key<number` compares"
                            )));
                        }
                        if values != "*" && values.split('|').any(|v| v.is_empty() || v == "*") {
                            return Err(RulesError(format!(
                                "condition {condition:?}: a value list has an empty or bare `*` entry"
                            )));
                        }
                        Ok(Condition {
                            key: key.to_string(),
                            test: Test::Values(
                                (values != "*")
                                    .then(|| values.split('|').map(str::to_string).collect()),
                            ),
                            negated,
                        })
                    })
                    .collect::<Result<Vec<_>, RulesError>>()?;
                let problem = if !when.iter().any(|c| !c.negated) {
                    Some("needs a condition that is not negated")
                } else if entry.schoolyard.is_none() && entry.sprint.is_none() {
                    Some("has no symbol for any map kind")
                } else if entry.point_as_short_line_mm.is_some() && entry.draw != DrawKind::Line {
                    Some("point_as_short_line_mm needs draw = \"line\"")
                } else {
                    None
                };
                if let Some(problem) = problem {
                    return Err(RulesError(format!("rule {:?} {problem}", entry.when)));
                }
                Ok(Rule {
                    when,
                    draw: entry.draw,
                    point_as_short_line_mm: entry.point_as_short_line_mm,
                    target: entry.target,
                    schoolyard: entry.schoolyard,
                    sprint: entry.sprint,
                })
            })
            .collect::<Result<_, _>>()
            .map(Rules)
    }

    /// Whether a rule with a symbol for `kind` holds for `tags` (used to filter the extract).
    pub fn wants(&self, tags: &[(String, String)], kind: MapKind) -> bool {
        let tags = tags.to_vec();
        self.0
            .iter()
            .filter(|r| match kind {
                MapKind::Schoolyard => r.schoolyard.is_some(),
                MapKind::Sprint => r.sprint.is_some(),
            })
            .any(|r| r.when.iter().all(|c| c.holds(&tags)))
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
            target: rule.target,
            matched,
        })
    }
}

/// Objects for the features within `buffer_m` of `map_area` (EPSG:25833) whose rule has `target`,
/// tagged with their provenance (`source`: `osm`, `geoportal-berlin/<layer>`) and `currency`.
/// Features from other sources carry `freemapper:*` tags (their `freemapper:source_id`, …) that
/// the objects take over; an OSM feature cannot set them.
#[allow(clippy::too_many_arguments)]
pub fn osm_objects(
    features: &[OsmFeature],
    rules: &Rules,
    target: Target,
    kind: MapKind,
    map_area: &geo_types::Polygon<f64>,
    buffer_m: f64,
    georef: &Georeferencing,
    symbol_set: &SymbolSet,
    source: &str,
    currency: NaiveDate,
) -> Result<Vec<MapObject>, GeorefError> {
    let projected = |p: &LatLon| {
        let q = georef.crs().forward(*p);
        geo_types::coord! { x: q.easting, y: q.northing }
    };
    let to_map = |p: &LatLon| -> Result<MapCoord, GeorefError> { georef.geographic_to_map(*p) };
    // Lines are clipped to the map area's bounding box plus buffer, like the land use objects, so
    // they never reach into the peripheral information.
    let clip = map_area.bounding_rect().map(|r| {
        geo_types::Rect::new(
            geo_types::coord! { x: r.min().x - buffer_m, y: r.min().y - buffer_m },
            geo_types::coord! { x: r.max().x + buffer_m, y: r.max().y + buffer_m },
        )
    });
    let min_length = f64::from(georef.scale_denominator) / 1000.0 * MIN_LINE_MM;
    let mut objects = Vec::new();
    for feature in features {
        let Some(found) = rules
            .symbol_for(&feature.tags, &feature.geometry, kind)
            .filter(|found| found.target == target)
        else {
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
        let pieces = match (&feature.geometry, found.draw) {
            (OsmGeometry::Line(points), Draw::Line)
                if clip.is_some_and(|clip| {
                    points.iter().map(projected).any(|c| {
                        c.x < clip.min().x
                            || c.x > clip.max().x
                            || c.y < clip.min().y
                            || c.y > clip.max().y
                    })
                }) =>
            {
                let clip = clip.expect("checked in the guard").to_polygon();
                let line = geo_types::LineString(points.iter().map(projected).collect());
                let mut pieces = Vec::new();
                for piece in clip.clip(&geo_types::MultiLineString(vec![line]), false) {
                    if Euclidean.length(&piece) < min_length {
                        continue;
                    }
                    let coords = piece
                        .coords()
                        .map(|c| {
                            georef.projected_to_map(ProjectedPoint {
                                easting: c.x,
                                northing: c.y,
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    pieces.push((MapObjectKind::Path, coords));
                }
                pieces
            }
            (OsmGeometry::Point(p), Draw::Point) => {
                vec![(MapObjectKind::Point { rotation_rad: 0.0 }, vec![to_map(p)?])]
            }
            (OsmGeometry::Point(p), Draw::ShortLine(mm)) => {
                let c = to_map(p)?;
                let half = (mm * 500.0).round() as i32;
                vec![(
                    MapObjectKind::Path,
                    vec![
                        MapCoord::new(c.x - half, c.y),
                        MapCoord::new(c.x + half, c.y),
                    ],
                )]
            }
            (OsmGeometry::Line(points), Draw::Line) => {
                let coords = points.iter().map(to_map).collect::<Result<Vec<_>, _>>()?;
                let closed = coords.len() > 3 && coords.first() == coords.last();
                let coords = if closed {
                    // A ring (fence or hedge around a yard) stays closed in OOM.
                    closed_ring(&coords.iter().map(|c| (c.x, c.y)).collect::<Vec<_>>())
                } else {
                    coords
                };
                vec![(MapObjectKind::Path, coords)]
            }
            (OsmGeometry::Line(points), Draw::Area) => {
                let ring = points
                    .iter()
                    .map(|p| to_map(p).map(|c| (c.x, c.y)))
                    .collect::<Result<Vec<_>, _>>()?;
                vec![(MapObjectKind::Path, closed_ring(&ring))]
            }
            _ => continue,
        };
        let own = |key: &str| {
            (source != "osm")
                .then(|| value(&feature.tags, key))
                .flatten()
                .map(str::to_string)
        };
        let mut tags = vec![
            ("freemapper:source".to_string(), source.to_string()),
            (
                "freemapper:source_id".to_string(),
                own("freemapper:source_id").unwrap_or_else(|| feature.id.clone()),
            ),
        ];
        if source == "osm" {
            tags.push(("freemapper:osm".to_string(), found.matched));
        }
        tags.push(("freemapper:currency".to_string(), currency.to_string()));
        for key in ["freemapper:alkis", "freemapper:size"] {
            if let Some(v) = own(key) {
                tags.push((key.to_string(), v));
            }
        }
        for (kind, coords) in pieces {
            objects.push(MapObject {
                symbol_id,
                kind,
                coords,
                tags: tags.clone(),
            });
        }
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
        let objects = osm_objects(
            features,
            &Rules::embedded(),
            Target::Candidates,
            kind,
            &f.area,
            20.0,
            &f.georef,
            &set,
            "osm",
            currency,
        )
        .unwrap();
        (objects, set)
    }

    #[test]
    fn embedded_rule_codes_exist_in_the_symbol_sets() {
        for rule in Rules::embedded().0.iter().chain(&Rules::berlin().0) {
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
    fn bare_star_in_a_value_list_is_rejected() {
        for list in ["a|*", "*|a", "a||b"] {
            let toml =
                format!("[[rule]]\nwhen = [\"k={list}\"]\ndraw = \"line\"\nsprint = \"515\"");
            assert!(Rules::from_toml(&toml).is_err(), "{list}");
        }
    }

    #[test]
    fn trailing_star_is_a_prefix() {
        let rules = Rules::from_toml(
            "[[rule]]\nwhen = [\"name=Wall*|Fence\"]\ndraw = \"line\"\nsprint = \"515\"",
        )
        .unwrap();
        for (name, expected) in [
            ("Wall", true),
            ("Wall, left", true),
            ("Fence", true),
            ("Fenced", false),
            ("Hall", false),
        ] {
            assert_eq!(
                rules.wants(&tags(&[("name", name)]), MapKind::Sprint),
                expected,
                "{name}"
            );
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
        assert!(Rules::embedded().wants(&tags(&[("barrier", "fence")]), MapKind::Schoolyard));
        assert!(!Rules::embedded().wants(&tags(&[("building", "yes")]), MapKind::Sprint));
    }

    fn way() -> OsmGeometry {
        OsmGeometry::Line(vec![at(0.0, 0.0), at(0.0005, 0.0005)])
    }

    /// Code and layer of the sprint symbol for a way with these tags.
    fn sprint_way(t: &[(&str, &str)]) -> Option<(String, Target)> {
        Rules::embedded()
            .symbol_for(&tags(t), &way(), MapKind::Sprint)
            .map(|m| (m.code, m.target))
    }

    #[test]
    fn width_below_is_numeric() {
        let rules = Rules::from_toml(
            "[[rule]]\nwhen = [\"highway=path\", \"width<1.5\"]\ndraw = \"line\"\nsprint = \"506\"",
        )
        .unwrap();
        for (width, holds) in [
            ("1.2", true),
            ("1,2 m", true),
            ("1.2m", true),
            ("2", false),
            ("1.5", false),
            ("narrow", false),
        ] {
            let found = rules.symbol_for(
                &tags(&[("highway", "path"), ("width", width)]),
                &way(),
                MapKind::Sprint,
            );
            assert_eq!(found.is_some(), holds, "width={width}");
        }
        let found = rules.symbol_for(&tags(&[("highway", "path")]), &way(), MapKind::Sprint);
        assert!(found.is_none(), "no width tag");
        assert!(
            Rules::from_toml("[[rule]]\nwhen = [\"width<x\"]\ndraw = \"line\"\nsprint = \"506\"")
                .is_err()
        );
    }

    #[test]
    fn unpaved_track_is_a_map_object() {
        let track = sprint_way(&[("highway", "track"), ("surface", "gravel")]);
        assert_eq!(track, Some(("505.2".to_string(), Target::Map)));
        let path = sprint_way(&[("highway", "footway"), ("surface", "dirt")]);
        assert_eq!(path, Some(("505.1".to_string(), Target::Map)));
    }

    #[test]
    fn narrow_or_faint_path_is_506() {
        let narrow = sprint_way(&[("highway", "track"), ("surface", "ground"), ("width", "1")]);
        assert_eq!(narrow, Some(("506".to_string(), Target::Map)));
        let faint = sprint_way(&[
            ("highway", "path"),
            ("surface", "earth"),
            ("trail_visibility", "bad"),
        ]);
        assert_eq!(faint, Some(("506".to_string(), Target::Map)));
    }

    #[test]
    fn paved_and_sidewalk_ways_are_dropped() {
        let asphalt = sprint_way(&[("highway", "path"), ("surface", "asphalt"), ("width", "1")]);
        assert_eq!(asphalt, None);
        let sidewalk = sprint_way(&[
            ("highway", "footway"),
            ("footway", "sidewalk"),
            ("surface", "gravel"),
        ]);
        assert_eq!(sidewalk, None);
        let crossing = sprint_way(&[("highway", "footway"), ("footway", "crossing")]);
        assert_eq!(crossing, None);
    }

    #[test]
    fn path_without_surface_is_a_candidate() {
        let path = sprint_way(&[("highway", "path")]);
        assert_eq!(path, Some(("505.1".to_string(), Target::Candidates)));
        assert!(Rules::embedded().wants(&tags(&[("highway", "track")]), MapKind::Sprint));
    }

    #[test]
    fn schoolyard_does_not_want_highways() {
        let track = tags(&[("highway", "track"), ("surface", "gravel")]);
        assert!(!Rules::embedded().wants(&track, MapKind::Schoolyard));
        assert!(Rules::embedded().wants(&track, MapKind::Sprint));
    }

    #[test]
    fn building_passages_and_area_ways_are_candidates() {
        for extra in [
            ("tunnel", "building_passage"),
            ("covered", "yes"),
            ("indoor", "yes"),
            ("area", "yes"),
        ] {
            let found = sprint_way(&[("highway", "footway"), ("surface", "fine_gravel"), extra]);
            assert_eq!(
                found,
                Some(("505.1".to_string(), Target::Candidates)),
                "{extra:?}"
            );
        }
    }

    #[test]
    fn unknown_unpaved_surface_is_a_candidate() {
        let found = sprint_way(&[("highway", "path"), ("surface", "pebblestone")]);
        assert_eq!(found, Some(("505.1".to_string(), Target::Candidates)));
        let paved = sprint_way(&[("highway", "path"), ("surface", "paving_stones")]);
        assert_eq!(paved, None);
    }

    #[test]
    fn unpaved_street_is_a_candidate() {
        let found = sprint_way(&[("highway", "residential"), ("surface", "sand")]);
        assert_eq!(found, Some(("505.2".to_string(), Target::Candidates)));
        assert_eq!(
            sprint_way(&[("highway", "residential"), ("surface", "asphalt")]),
            None
        );
    }

    #[test]
    fn nonsense_widths_do_not_count() {
        for width in ["-1", "0", "-inf", "inf", "NaN"] {
            let found = sprint_way(&[("highway", "path"), ("surface", "dirt"), ("width", width)]);
            assert_eq!(
                found,
                Some(("505.1".to_string(), Target::Map)),
                "width={width}"
            );
        }
    }

    #[test]
    fn comparison_keys_are_rejected() {
        for condition in ["width<=1.5", "width>2", "width>=2"] {
            let toml =
                format!("[[rule]]\nwhen = [\"{condition}\"]\ndraw = \"line\"\nsprint = \"506\"");
            assert!(Rules::from_toml(&toml).is_err(), "{condition}");
        }
    }

    #[test]
    fn lines_are_clipped_to_the_map_area_and_buffer() {
        let f = fixture();
        // From the centre 500 m east: clipped at 60 m (area) + 20 m (buffer).
        let track = line(
            "way/9",
            &[("highway", "track"), ("surface", "gravel")],
            vec![SCHOOL, at(0.0, 0.0074)],
        );
        let set = MapKind::Sprint.symbol_set();
        let currency = "2026-11-06".parse().unwrap();
        let objects = osm_objects(
            &[track],
            &Rules::embedded(),
            Target::Map,
            MapKind::Sprint,
            &f.area,
            20.0,
            &f.georef,
            &set,
            "osm",
            currency,
        )
        .unwrap();
        assert_eq!(objects.len(), 1);
        let (e, n) = (
            f.georef.ref_projected.easting,
            f.georef.ref_projected.northing,
        );
        for c in &objects[0].coords {
            let p = f.georef.map_to_projected(*c);
            assert!(
                (p.easting - e).abs() <= 80.1 && (p.northing - n).abs() <= 80.1,
                "{p:?}"
            );
        }
        let last = f
            .georef
            .map_to_projected(*objects[0].coords.last().unwrap());
        assert!((last.easting - e - 80.0).abs() < 0.5, "{last:?}");
    }

    #[test]
    fn schoolyard_has_no_path_rules() {
        for t in [
            vec![("highway", "track"), ("surface", "gravel")],
            vec![("highway", "path")],
        ] {
            let found = Rules::embedded().symbol_for(&tags(&t), &way(), MapKind::Schoolyard);
            assert_eq!(found, None, "{t:?}");
        }
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
        assert!(
            objects(std::slice::from_ref(&hydrant), MapKind::Sprint)
                .0
                .is_empty()
        );
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
    fn closed_fence_becomes_a_closed_line() {
        let ring = line("way/5", &[("barrier", "fence")], square());
        let (objects, _) = objects(&[ring], MapKind::Sprint);
        assert_eq!(objects[0].coords.len(), 5);
        assert_eq!(objects[0].coords[4].flags, FLAG_HOLE | FLAG_CLOSE);
    }

    #[test]
    fn underground_hydrant_is_not_a_candidate() {
        let mut hydrant = point(
            "node/9669755236",
            &[
                ("emergency", "fire_hydrant"),
                ("fire_hydrant:type", "underground"),
            ],
        );
        assert!(
            objects(std::slice::from_ref(&hydrant), MapKind::Schoolyard)
                .0
                .is_empty()
        );
        hydrant.tags = tags(&[
            ("emergency", "fire_hydrant"),
            ("fire_hydrant:type", "pillar"),
        ]);
        assert_eq!(objects(&[hydrant], MapKind::Schoolyard).0.len(), 1);
    }

    #[test]
    fn invalid_rules_are_rejected() {
        let rule = |body: &str| Rules::from_toml(&format!("[[rule]]\n{body}"));
        assert!(
            rule("when = []\ndraw = \"point\"\nsprint = \"418\"").is_err(),
            "empty when"
        );
        assert!(
            rule("when = [\"natural=tree\"]\ndraw = \"point\"").is_err(),
            "no symbol"
        );
        assert!(
            rule("when = [\"natural=tree\"]\ndraw = \"point\"\npoint_as_short_line_mm = 1.0\nsprint = \"418\"").is_err(),
            "short line on a point rule"
        );
        assert!(
            rule("when = [\"a!=b\"]\ndraw = \"point\"\nsprint = \"418\"").is_err(),
            "only negated"
        );
        assert!(rule("when = [\"a=*\", \"b!=c\"]\ndraw = \"point\"\nsprint = \"418\"").is_ok());
    }

    #[test]
    fn embedded_rule_symbols_fit_their_draw_kind() {
        for rule in Rules::embedded().0.iter().chain(&Rules::berlin().0) {
            for (code, kind) in [
                (&rule.schoolyard, MapKind::Schoolyard),
                (&rule.sprint, MapKind::Sprint),
            ] {
                let Some(code) = code else { continue };
                let set = kind.symbol_set();
                let symbol_type = set.symbol_type(set.id_for_code(code).unwrap());
                let fits: &[u32] = match rule.draw {
                    DrawKind::Point => &[1],
                    DrawKind::Line => &[2, 16],
                    DrawKind::Area => &[4, 16],
                };
                assert!(
                    symbol_type.is_some_and(|t| fits.contains(&t)),
                    "{code}: type {symbol_type:?} for {:?}",
                    rule.draw
                );
            }
        }
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

    fn sprint_code(t: &[(&str, &str)], geometry: &OsmGeometry) -> Option<(String, Draw)> {
        Rules::embedded()
            .symbol_for(&tags(t), geometry, MapKind::Sprint)
            .map(|m| (m.code, m.draw))
    }

    fn ring() -> OsmGeometry {
        OsmGeometry::Line(square())
    }

    #[test]
    fn water_tags_are_stored() {
        let rules = Rules::embedded();
        for kind in [MapKind::Schoolyard, MapKind::Sprint] {
            for t in [
                &[("natural", "water")][..],
                &[("waterway", "riverbank")],
                &[("landuse", "reservoir")],
                &[("landuse", "basin")],
                &[("leisure", "swimming_pool")],
                &[("amenity", "fountain")],
                &[("man_made", "water_well")],
                &[("natural", "spring")],
                &[("waterway", "stream")],
                &[("waterway", "drain")],
                &[("waterway", "ditch")],
            ] {
                assert!(rules.wants(&tags(t), kind), "{t:?} {kind:?}");
            }
        }
        // Drinking water is a schoolyard symbol, bridges are sprint only.
        let drinking = tags(&[("amenity", "drinking_water")]);
        assert!(rules.wants(&drinking, MapKind::Schoolyard));
        assert!(!rules.wants(&drinking, MapKind::Sprint));
        let bridge = tags(&[("man_made", "bridge")]);
        assert!(rules.wants(&bridge, MapKind::Sprint));
        assert!(!rules.wants(&bridge, MapKind::Schoolyard));
        // Rivers and canals are lines of the centre only: no rule.
        assert!(!rules.wants(&tags(&[("waterway", "river")]), MapKind::Sprint));
    }

    #[test]
    fn water_areas_are_candidates() {
        for t in [
            &[("natural", "water"), ("water", "pond")][..],
            &[("waterway", "riverbank")],
            &[("landuse", "reservoir")],
            &[("landuse", "basin")],
            &[("leisure", "swimming_pool")],
            &[("amenity", "fountain")],
        ] {
            assert_eq!(
                sprint_code(t, &ring()),
                Some(("301".to_string(), Draw::Area)),
                "{t:?}"
            );
        }
        // A retention basin is dry most of the time.
        assert_eq!(
            sprint_code(&[("landuse", "basin"), ("basin", "retention")], &ring()),
            None
        );
        // A covered or tunnelled water body is not on the surface.
        assert_eq!(
            sprint_code(&[("natural", "water"), ("covered", "yes")], &ring()),
            None
        );
        assert_eq!(
            sprint_code(&[("natural", "water"), ("tunnel", "culvert")], &ring()),
            None
        );
        // An unclosed way is no area.
        assert_eq!(sprint_code(&[("natural", "water")], &way()), None);
        let (objects, set) = objects(
            &[line("w1", &[("natural", "water")], square())],
            MapKind::Sprint,
        );
        assert_eq!(objects.len(), 1);
        assert_eq!(Some(objects[0].symbol_id), set.id_for_code("301"));
    }

    #[test]
    fn indoor_pools_are_skipped() {
        for extra in [
            ("indoor", "yes"),
            ("covered", "yes"),
            ("location", "indoor"),
            ("location", "underground"),
            ("location", "roof"),
        ] {
            assert_eq!(
                sprint_code(&[("leisure", "swimming_pool"), extra], &ring()),
                None,
                "{extra:?}"
            );
        }
        assert_eq!(
            sprint_code(
                &[("leisure", "swimming_pool"), ("access", "private")],
                &ring()
            ),
            Some(("301".to_string(), Draw::Area))
        );
    }

    #[test]
    fn streams_and_ditches_are_lines() {
        let l = way();
        let code = |t: &[(&str, &str)]| sprint_code(t, &l).map(|(c, _)| c);
        assert_eq!(code(&[("waterway", "stream")]), Some("305".into()));
        assert_eq!(code(&[("waterway", "drain")]), Some("305".into()));
        assert_eq!(code(&[("waterway", "ditch")]), Some("306".into()));
        // A stream that falls dry is a ditch.
        assert_eq!(
            code(&[("waterway", "stream"), ("intermittent", "yes")]),
            Some("306".into())
        );
        let (objects, set) = objects(
            &[line(
                "s1",
                &[("waterway", "stream")],
                vec![at(0.0, 0.0), at(0.0004, 0.0004)],
            )],
            MapKind::Schoolyard,
        );
        assert_eq!(objects.len(), 1);
        assert_eq!(Some(objects[0].symbol_id), set.id_for_code("305"));
    }

    #[test]
    fn stream_tagged_tunnel_no_is_kept() {
        let l = way();
        let code = |t: &[(&str, &str)]| sprint_code(t, &l).map(|(c, _)| c);
        assert_eq!(
            code(&[("waterway", "stream"), ("tunnel", "no")]),
            Some("305".into())
        );
        assert_eq!(
            code(&[("waterway", "ditch"), ("tunnel", "no")]),
            Some("306".into())
        );
        assert_eq!(
            sprint_code(&[("natural", "water"), ("tunnel", "no")], &ring()),
            Some(("301".to_string(), Draw::Area))
        );
    }

    #[test]
    fn culverts_are_skipped() {
        let l = way();
        for t in ["culvert", "yes", "flooded", "building_passage", "canal"] {
            for waterway in ["stream", "drain", "ditch"] {
                assert_eq!(
                    sprint_code(&[("waterway", waterway), ("tunnel", t)], &l),
                    None,
                    "{waterway} {t}"
                );
            }
        }
    }

    #[test]
    fn fountain_node_is_a_small_fountain() {
        let p = OsmGeometry::Point(SCHOOL);
        assert_eq!(
            sprint_code(&[("amenity", "fountain")], &p),
            Some(("311".to_string(), Draw::Point))
        );
        assert_eq!(
            sprint_code(&[("man_made", "water_well")], &p),
            Some(("311".to_string(), Draw::Point))
        );
        assert_eq!(
            sprint_code(&[("natural", "spring")], &p),
            Some(("312".to_string(), Draw::Point))
        );
        let drinking = [("amenity", "drinking_water")];
        assert_eq!(
            Rules::embedded()
                .symbol_for(&tags(&drinking), &p, MapKind::Schoolyard)
                .map(|m| m.code),
            Some("313".to_string())
        );
        assert_eq!(sprint_code(&drinking, &p), None);
        // The basin of a fountain drawn as an area stays 301.
        assert_eq!(
            sprint_code(&[("amenity", "fountain")], &ring()),
            Some(("301".to_string(), Draw::Area))
        );
        let (objects, set) = objects(&[point("f1", &[("amenity", "fountain")])], MapKind::Sprint);
        assert_eq!(objects.len(), 1);
        assert_eq!(Some(objects[0].symbol_id), set.id_for_code("311"));
    }

    #[test]
    fn river_centre_lines_are_not_candidates() {
        let l = way();
        for waterway in ["river", "canal", "tidal_channel"] {
            for kind in [MapKind::Sprint, MapKind::Schoolyard] {
                assert!(
                    Rules::embedded()
                        .symbol_for(&tags(&[("waterway", waterway)]), &l, kind)
                        .is_none(),
                    "{waterway}"
                );
            }
        }
    }

    #[test]
    fn bridges_are_paved_candidates() {
        assert_eq!(
            sprint_code(&[("man_made", "bridge")], &ring()),
            Some(("501.5".to_string(), Draw::Area))
        );
        // Not on the schoolyard map, and not as a line.
        assert!(
            Rules::embedded()
                .symbol_for(
                    &tags(&[("man_made", "bridge")]),
                    &ring(),
                    MapKind::Schoolyard
                )
                .is_none()
        );
        assert_eq!(sprint_code(&[("man_made", "bridge")], &way()), None);
        let (objects, set) = objects(
            &[line("b1", &[("man_made", "bridge")], square())],
            MapKind::Sprint,
        );
        assert_eq!(objects.len(), 1);
        assert_eq!(Some(objects[0].symbol_id), set.id_for_code("501.5"));
    }

    #[test]
    fn water_is_a_schoolyard_symbol_too() {
        let found = |t: &[(&str, &str)], g: &OsmGeometry| {
            Rules::embedded()
                .symbol_for(&tags(t), g, MapKind::Schoolyard)
                .map(|m| (m.code, m.draw))
        };
        assert_eq!(
            found(&[("natural", "water")], &ring()),
            Some(("301".to_string(), Draw::Area))
        );
        assert_eq!(
            found(&[("amenity", "fountain")], &OsmGeometry::Point(SCHOOL)),
            Some(("311".to_string(), Draw::Point))
        );
        let (objects, set) = objects(
            &[
                line("w1", &[("natural", "water")], square()),
                point("f1", &[("amenity", "fountain")]),
            ],
            MapKind::Schoolyard,
        );
        let ids: Vec<_> = objects.iter().map(|o| Some(o.symbol_id)).collect();
        assert_eq!(ids, vec![set.id_for_code("301"), set.id_for_code("311")]);
    }
}
