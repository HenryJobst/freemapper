//! OSM features → candidate objects for the candidates template map (spec §4.5).

use std::collections::HashMap;

use chrono::NaiveDate;
use geo::{BooleanOps as _, BoundingRect as _, Distance as _, Euclidean, Length as _};
use serde::Deserialize;

use crate::crs::{LatLon, ProjectedPoint};
use crate::georef::{GeorefError, Georeferencing};
use crate::omap::{MapCoord, MapObject, MapObjectKind, SymbolSet, closed_ring};
use crate::project::MapKind;
use crate::sources::osm::{OsmFeature, OsmGeometry, OsmPolygon, Tags};

const EMBEDDED_RULES: &str = include_str!("../../../../assets/rules/osm-candidates.toml");
const STATE_RULES: &str = include_str!("../../../../assets/rules/state-candidates.toml");
const BERLIN_RULES: &str = include_str!("../../../../assets/rules/berlin-candidates.toml");
/// Line pieces shorter than this on paper are left over from clipping at the buffer edge.
pub(crate) const MIN_LINE_MM: f64 = 1.0;

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
    /// A way drawn as an area: a strip along it of this width on the ground (metres), or of its
    /// `width` tag (see [`CORRIDOR_WIDTH_RANGE_M`]).
    Corridor(f64),
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
    corridor_m: Option<f64>,
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
    /// `key>=number`: the value is a number of at least this.
    AtLeast(f64),
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
    corridor_m: Option<f64>,
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
            (Some(v), Test::AtLeast(limit)) => number(v).is_some_and(|n| n >= *limit),
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

    /// The ALKIS vereinfacht rules (`assets/rules/state-candidates.toml`): `alkis:*` tags only.
    pub fn states() -> Self {
        Self::from_toml(STATE_RULES).expect("embedded state rules are valid")
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
                        // A comparison key is a plain tag name: `name=a>=b` is a value match and
                        // `a|b>=c` (a value list as key) is rejected below.
                        let compare = if let Some((key, limit)) = condition
                            .split_once(">=")
                            .filter(|(key, _)| !key.contains(['=', '|']))
                        {
                            Some((key, limit, Test::AtLeast as fn(f64) -> Test))
                        } else if !condition.contains('=') {
                            condition
                                .split_once('<')
                                .map(|(key, limit)| (key, limit, Test::Below as fn(f64) -> Test))
                        } else {
                            None
                        };
                        if let Some((key, limit, test)) = compare {
                            let limit = number(limit).ok_or_else(|| {
                                RulesError(format!(
                                    "condition {condition:?}: {limit:?} is no number"
                                ))
                            })?;
                            return Ok(Condition {
                                key: key.to_string(),
                                test: test(limit),
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
                                "condition {condition:?}: only `key<number` and `key>=number` compare"
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
                } else if entry.corridor_m.is_some() && entry.draw != DrawKind::Line {
                    Some("corridor_m needs draw = \"line\"")
                } else if entry.corridor_m.is_some_and(|w| !(w.is_finite() && w > 0.0)) {
                    Some("corridor_m must be a positive number")
                } else if entry.corridor_m.is_some() && entry.point_as_short_line_mm.is_some() {
                    Some("corridor_m and point_as_short_line_mm exclude each other")
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
                    corridor_m: entry.corridor_m,
                    target: entry.target,
                    schoolyard: entry.schoolyard,
                    sprint: entry.sprint,
                })
            })
            .collect::<Result<_, _>>()
            .map(Rules)
    }

    /// The symbol codes of the rules drawn as points, both map kinds.
    #[cfg(test)]
    pub(crate) fn point_codes(&self) -> Vec<String> {
        self.0
            .iter()
            .filter(|r| r.draw == DrawKind::Point)
            .flat_map(|r| [r.schoolyard.clone(), r.sprint.clone()])
            .flatten()
            .collect()
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
                (DrawKind::Line, OsmGeometry::Line(_)) => match rule.corridor_m {
                    Some(width) => Draw::Corridor(width),
                    None => Draw::Line,
                },
                (DrawKind::Line, OsmGeometry::Point(_)) => {
                    Draw::ShortLine(rule.point_as_short_line_mm?)
                }
                (DrawKind::Area, OsmGeometry::Line(_)) if closed => Draw::Area,
                (DrawKind::Area, OsmGeometry::Area(_)) => Draw::Area,
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
    let projected_polygon = |polygon: &OsmPolygon| {
        let ring =
            |points: &Vec<LatLon>| geo_types::LineString(points.iter().map(projected).collect());
        geo_types::Polygon::new(
            ring(&polygon.outer),
            polygon.holes.iter().map(ring).collect(),
        )
    };
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
    // The cadastre cuts a wall into short segments; the length limit holds for the whole wall.
    let joined;
    let features = if source == "osm" {
        features
    } else {
        joined = join_touching_lines(features, rules, target, kind, symbol_set, georef, source);
        &joined
    };
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
            OsmGeometry::Area(polygons) => polygons
                .iter()
                .map(|polygon| Euclidean.distance(&projected_polygon(polygon), map_area))
                .fold(f64::INFINITY, f64::min),
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
            // A cadastre or survey line shorter than 1 mm on paper is not worth a candidate (the
            // clipped pieces above are held to the same limit).
            (OsmGeometry::Line(points), Draw::Line)
                if source != "osm"
                    && Euclidean.length(&geo_types::LineString(
                        points.iter().map(projected).collect(),
                    )) < min_length =>
            {
                continue;
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
            (OsmGeometry::Line(points), Draw::Corridor(default_width)) => {
                let width = value(&feature.tags, "width")
                    .and_then(number)
                    .unwrap_or(default_width)
                    .clamp(CORRIDOR_WIDTH_RANGE_M.0, CORRIDOR_WIDTH_RANGE_M.1);
                let line: Vec<_> = points.iter().map(projected).collect();
                let mut pieces = Vec::new();
                for polygon in corridor(&line, width) {
                    let mut coords = Vec::new();
                    for ring in std::iter::once(polygon.exterior()).chain(polygon.interiors()) {
                        let ring = ring
                            .coords()
                            .map(|c| {
                                georef
                                    .projected_to_map(ProjectedPoint {
                                        easting: c.x,
                                        northing: c.y,
                                    })
                                    .map(|c| (c.x, c.y))
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        coords.extend(closed_ring(&ring));
                    }
                    pieces.push((MapObjectKind::Path, coords));
                }
                pieces
            }
            (OsmGeometry::Line(points), Draw::Area) => {
                let ring = points
                    .iter()
                    .map(|p| to_map(p).map(|c| (c.x, c.y)))
                    .collect::<Result<Vec<_>, _>>()?;
                vec![(MapObjectKind::Path, closed_ring(&ring))]
            }
            // A relation: one object per polygon, the outer ring first, then its holes.
            (OsmGeometry::Area(polygons), Draw::Area) => {
                let mut pieces = Vec::new();
                for polygon in polygons {
                    let mut coords = Vec::new();
                    for ring in std::iter::once(&polygon.outer).chain(&polygon.holes) {
                        let ring = ring
                            .iter()
                            .map(|p| to_map(p).map(|c| (c.x, c.y)))
                            .collect::<Result<Vec<_>, _>>()?;
                        coords.extend(closed_ring(&ring));
                    }
                    pieces.push((MapObjectKind::Path, coords));
                }
                pieces
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
        // A feature with a date of its own (Straßenbefahrung) is as old as that.
        tags.push((
            "freemapper:currency".to_string(),
            own("freemapper:currency").unwrap_or_else(|| currency.to_string()),
        ));
        let codes = crate::build::berlin::SURVEY_CODES.map(|code| format!("berlin:{code}"));
        for key in [
            "berlin:layer",
            "alkis:type",
            "tree:dataset",
            "freemapper:also",
            "freemapper:alkis",
            "freemapper:size",
            "freemapper:baum",
            "freemapper:crown_m",
            "freemapper:currency_guessed",
        ]
        .into_iter()
        .chain(codes.iter().map(String::as_str))
        {
            if let Some(v) = own(key) {
                tags.push((key.to_string(), v));
            }
        }
        // An OSM tree may give its crown diameter (metres) for the point generalization.
        if source == "osm"
            && let Some(crown) = value(&feature.tags, "diameter_crown").and_then(number)
        {
            tags.push(("freemapper:crown_m".to_string(), crown.to_string()));
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

/// The full width of a [`Draw::Corridor`] on the ground (metres) is held within these limits: a
/// `width` tag below the lower one still leaves the 1 m footprint of the norm's smallest area
/// (ISSprOM 520, 522: minimum width 0.25 mm at 1:4000), and an absurd one the upper one.
pub(crate) const CORRIDOR_WIDTH_RANGE_M: (f64, f64) = (1.5, 12.0);

/// The strip along `points` (projected, metres) of the full width `width_m`, ends cut straight so
/// that it stops where the way ends (a passage ends at the street).
fn corridor(points: &[geo_types::Coord<f64>], width_m: f64) -> geo_types::MultiPolygon<f64> {
    use geo::Buffer as _;
    use geo::algorithm::buffer::{BufferStyle, LineCap};
    let style = BufferStyle::new(width_m / 2.0).line_cap(LineCap::Butt);
    geo_types::LineString(points.to_vec()).buffer_with_style(style)
}

/// Segments whose ends are within this distance (ground metres) touch.
const JOIN_TOUCH_M: f64 = 0.1;
/// Segments join when the bend between them is at most about 30 degrees (the cosine).
const JOIN_MIN_COS: f64 = 0.866;

/// Line symbols whose drawing has a side (ticks on the lower side), so their direction matters.
const SIDED_CODES: [&str; 3] = ["104", "201", "513.2"];
/// NAS codes of an edge, whose direction says on which side the object lies: wall edges,
/// retaining wall edges and hedge edges (right and left).
const SIDED_EDGES: [(&str, &[&str]); 2] = [
    ("alkis:bauwerksfunktion", &["1701", "1702", "1721", "1722"]),
    ("alkis:bewuchs", &["1101", "1102"]),
];

/// The side of a line feature drawn with `code`: `None` when its direction says nothing (it may
/// be reversed), else its NAS edge code (`""` without one). Lines with a side join only lines
/// with the same side, and only the same way round.
fn line_side(tags: &Tags, code: &str) -> Option<String> {
    let edge = SIDED_EDGES.iter().find_map(|(key, codes)| {
        value(tags, key)
            .filter(|v| codes.contains(v))
            .map(str::to_string)
    });
    edge.or_else(|| SIDED_CODES.contains(&code).then(String::new))
}

/// Chains the line features that the rules draw as lines with the same symbol and touch end to
/// end (within [`JOIN_TOUCH_M`], bend up to about 30 degrees) into one line each. A line with a
/// side ([`line_side`]) is never turned round and joins only lines with the same side, so a
/// later tick side stays right. The chain keeps the tags of its first segment and names the
/// other segments in `freemapper:also` (`<source>/<source_id>`). A chain of dated segments (the
/// survey) is as old as its oldest segment, and guessed when one of them is. Other features,
/// and lines that no rule draws as lines, are passed on unchanged.
fn join_touching_lines(
    features: &[OsmFeature],
    rules: &Rules,
    target: Target,
    kind: MapKind,
    symbol_set: &SymbolSet,
    georef: &Georeferencing,
    source: &str,
) -> Vec<OsmFeature> {
    let projected = |p: &LatLon| {
        let q = georef.crs().forward(*p);
        (q.easting, q.northing)
    };
    // The symbol and the side of each segment that is a plain open line.
    let (symbols, sides): (Vec<Option<u32>>, Vec<Option<String>>) = features
        .iter()
        .map(|f| {
            let OsmGeometry::Line(points) = &f.geometry else {
                return (None, None);
            };
            if points.len() < 2 || points.first() == points.last() {
                return (None, None);
            }
            let Some(found) = rules
                .symbol_for(&f.tags, &f.geometry, kind)
                .filter(|m| m.target == target && matches!(m.draw, Draw::Line))
            else {
                return (None, None);
            };
            (
                symbol_set.id_for_code(&found.code),
                line_side(&f.tags, &found.code),
            )
        })
        .unzip();
    let cell = |(x, y): (f64, f64)| {
        (
            (x / JOIN_TOUCH_M).floor() as i64,
            (y / JOIN_TOUCH_M).floor() as i64,
        )
    };
    // The ends of the segments by symbol and cell: (segment, is the last point).
    let mut ends: HashMap<(u32, i64, i64), Vec<(usize, bool)>> = HashMap::new();
    for (i, f) in features.iter().enumerate() {
        let (Some(symbol), OsmGeometry::Line(points)) = (symbols[i], &f.geometry) else {
            continue;
        };
        for (at_end, p) in [(false, points[0]), (true, points[points.len() - 1])] {
            let (cx, cy) = cell(projected(&p));
            ends.entry((symbol, cx, cy)).or_default().push((i, at_end));
        }
    }
    // The unused segment that continues `chain` (whose first segment is `first`) at its last
    // point, as (index, reversed). A chain that is `turned` grows at its start: there a segment
    // with a side must end where the chain starts.
    let next = |chain: &[LatLon], used: &[bool], symbol: u32, first: usize, turned: bool| {
        let last = projected(chain.last()?);
        let before = projected(&chain[chain.len() - 2]);
        let heading = (last.0 - before.0, last.1 - before.1);
        let (cx, cy) = cell(last);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for &(j, at_end) in ends.get(&(symbol, cx + dx, cy + dy)).into_iter().flatten() {
                    let OsmGeometry::Line(points) = &features[j].geometry else {
                        continue;
                    };
                    let (touching, second) = if at_end {
                        (points[points.len() - 1], points[points.len() - 2])
                    } else {
                        (points[0], points[1])
                    };
                    let (t, s) = (projected(&touching), projected(&second));
                    if used[j] || (t.0 - last.0).hypot(t.1 - last.1) > JOIN_TOUCH_M {
                        continue;
                    }
                    if sides[j] != sides[first] || (sides[j].is_some() && at_end != turned) {
                        continue;
                    }
                    let out = (s.0 - t.0, s.1 - t.1);
                    let norm = heading.0.hypot(heading.1) * out.0.hypot(out.1);
                    if norm > 0.0 && (heading.0 * out.0 + heading.1 * out.1) / norm >= JOIN_MIN_COS
                    {
                        return Some((j, at_end));
                    }
                }
            }
        }
        None::<(usize, bool)>
    };
    let mut used = vec![false; features.len()];
    let mut out = Vec::with_capacity(features.len());
    for (i, f) in features.iter().enumerate() {
        if used[i] {
            continue;
        }
        used[i] = true;
        let (Some(symbol), OsmGeometry::Line(points)) = (symbols[i], &f.geometry) else {
            out.push(f.clone());
            continue;
        };
        let mut chain = points.clone();
        let mut members = vec![i];
        // Grow at the end, turn the chain round, grow at the other end.
        for turned in [false, true] {
            while let Some((j, reversed)) = next(&chain, &used, symbol, i, turned) {
                used[j] = true;
                members.push(j);
                let OsmGeometry::Line(more) = &features[j].geometry else {
                    unreachable!("indexed as a line");
                };
                let more: Vec<LatLon> = if reversed {
                    more.iter().rev().copied().collect()
                } else {
                    more.clone()
                };
                chain.extend(more.into_iter().skip(1));
            }
            chain.reverse();
        }
        out.push(OsmFeature {
            id: f.id.clone(),
            tags: chain_tags(features, &members, source),
            geometry: OsmGeometry::Line(chain),
        });
    }
    out
}

/// The tags of a chain of `members` (indices into `features`, the first one first): those of
/// the first segment, the others named in `freemapper:also`, the oldest `freemapper:currency` of
/// the segments and `freemapper:currency_guessed=yes` when one of them is guessed.
fn chain_tags(features: &[OsmFeature], members: &[usize], source: &str) -> Tags {
    let first = &features[members[0]];
    let mut tags = first.tags.clone();
    if members.len() == 1 {
        return tags;
    }
    for &j in &members[1..] {
        let other = &features[j];
        let id = value(&other.tags, "freemapper:source_id").unwrap_or(&other.id);
        crate::build::dedup::add_also(&mut tags, &format!("{source}/{id}"));
    }
    let segments = || members.iter().map(|&j| &features[j].tags);
    let oldest = segments()
        .filter_map(|t| value(t, "freemapper:currency")?.parse::<NaiveDate>().ok())
        .min();
    if let Some(oldest) = oldest {
        tags.retain(|(k, _)| k != "freemapper:currency");
        tags.push(("freemapper:currency".to_string(), oldest.to_string()));
    }
    let guessed = segments().any(|t| value(t, "freemapper:currency_guessed") == Some("yes"));
    if guessed && value(&tags, "freemapper:currency_guessed").is_none() {
        tags.push(("freemapper:currency_guessed".to_string(), "yes".to_string()));
    }
    tags
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
    fn short_cadastre_segments_of_one_wall_are_joined() {
        // Three 2 m wall segments end to end at 1:4000, where 1 mm on paper is 4 m: each alone is
        // too short, the 6 m wall is not. A segment beside them that does not touch, a bend of
        // 90 degrees and a segment of another symbol stay apart.
        let date = "2026-11-07".parse().unwrap();
        let georef = Georeferencing::new(SCHOOL, 4000, date).unwrap();
        let (e, n) = (georef.ref_projected.easting, georef.ref_projected.northing);
        let area = geo_types::Polygon::new(
            vec![
                (e - 60.0, n - 60.0),
                (e + 60.0, n - 60.0),
                (e + 60.0, n + 60.0),
                (e - 60.0, n + 60.0),
                (e - 60.0, n - 60.0),
            ]
            .into(),
            vec![],
        );
        let step = 2.0 * 1.475e-5; // 2 m of longitude
        let segment = |id: &str, name: &str, from: f64, to: f64, dlat: f64| {
            line(
                id,
                &[("berlin:layer", "bauwerke-linien"), ("berlin:bezbwf", name)],
                vec![at(dlat, from * step), at(dlat, to * step)],
            )
        };
        let features = [
            // Out of order and one reversed: the chain does not depend on the file order.
            segment("b", "Mauer", 1.0, 2.0, 0.0),
            segment("a", "Mauer", 0.0, 1.0, 0.0),
            segment("c", "Mauer", 3.0, 2.0, 0.0),
            // Far away, 2 m long: dropped.
            segment("lone", "Mauer", 0.0, 1.0, 0.0002),
            // Touches the end of the wall at a right angle: not the same wall, 2 m, dropped.
            line(
                "bend",
                &[
                    ("berlin:layer", "bauwerke-linien"),
                    ("berlin:bezbwf", "Mauer"),
                ],
                vec![at(0.0, 3.0 * step), at(1.8e-5, 3.0 * step)],
            ),
        ];
        let set = MapKind::Sprint.symbol_set();
        let found = osm_objects(
            &features,
            &Rules::berlin(),
            Target::Candidates,
            MapKind::Sprint,
            &area,
            20.0,
            &georef,
            &set,
            "geoportal-berlin/bauwerke-linien",
            "2026-11-06".parse().unwrap(),
        )
        .unwrap();
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].coords.len(), 4);
    }

    #[test]
    fn joined_survey_line_names_all_segments_and_the_oldest_date() {
        // A survey wall of three 2 m pieces surveyed on different days, one with a guessed date.
        let date = "2026-11-07".parse().unwrap();
        let georef = Georeferencing::new(SCHOOL, 4000, date).unwrap();
        let (e, n) = (georef.ref_projected.easting, georef.ref_projected.northing);
        let area = geo_types::Polygon::new(
            vec![
                (e - 60.0, n - 60.0),
                (e + 60.0, n - 60.0),
                (e + 60.0, n + 60.0),
                (e - 60.0, n + 60.0),
                (e - 60.0, n - 60.0),
            ]
            .into(),
            vec![],
        );
        let step = 2.0 * 1.475e-5; // 2 m of longitude
        let piece = |id: &str, currency: &str, guessed: bool, from: f64, to: f64| {
            let mut t = vec![
                ("berlin:layer", "bh_mauer"),
                ("freemapper:source_id", id),
                ("freemapper:currency", currency),
            ];
            if guessed {
                t.push(("freemapper:currency_guessed", "yes"));
            }
            line(id, &t, vec![at(0.0, from * step), at(0.0, to * step)])
        };
        let features = [
            piece("Li_20150413_1", "2015-04-13", false, 0.0, 1.0),
            piece("Li_20141006_2", "2014-10-06", false, 1.0, 2.0),
            piece("Li_x_3", "2014-01-01", true, 2.0, 3.0),
        ];
        let set = MapKind::Sprint.symbol_set();
        let joined = |features: &[OsmFeature]| {
            osm_objects(
                features,
                &Rules::berlin(),
                Target::Candidates,
                MapKind::Sprint,
                &area,
                20.0,
                &georef,
                &set,
                "geoportal-berlin/sb-mauer",
                "2026-11-06".parse().unwrap(),
            )
            .unwrap()
        };
        let found = joined(&features);
        assert_eq!(found.len(), 1, "{found:?}");
        let tag = |key: &str| value(&found[0].tags, key).map(str::to_string);
        assert_eq!(
            tag("freemapper:source_id").as_deref(),
            Some("Li_20150413_1")
        );
        let also = tag("freemapper:also").expect("the other segments are named");
        let mut also: Vec<&str> = also.split("; ").collect();
        also.sort();
        assert_eq!(
            also,
            [
                "geoportal-berlin/sb-mauer/Li_20141006_2",
                "geoportal-berlin/sb-mauer/Li_x_3"
            ]
        );
        // The oldest date stands for the line; one guessed piece marks it as guessed.
        assert_eq!(tag("freemapper:currency").as_deref(), Some("2014-01-01"));
        assert_eq!(tag("freemapper:currency_guessed").as_deref(), Some("yes"));
        // Only real dates: the oldest, no flag.
        let features = [
            piece("Li_20150413_1", "2015-04-13", false, 0.0, 1.0),
            piece("Li_20141006_2", "2014-10-06", false, 1.0, 2.0),
            piece("Li_20150502_3", "2015-05-02", false, 2.0, 3.0),
        ];
        let found = joined(&features);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(
            value(&found[0].tags, "freemapper:currency"),
            Some("2014-10-06")
        );
        assert_eq!(value(&found[0].tags, "freemapper:currency_guessed"), None);
    }

    #[test]
    fn embedded_rule_codes_exist_in_the_symbol_sets() {
        for rule in Rules::embedded()
            .0
            .iter()
            .chain(&Rules::berlin().0)
            .chain(&Rules::states().0)
        {
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
    fn indoor_and_area_ways_are_candidates() {
        for extra in [("indoor", "yes"), ("area", "yes")] {
            let found = sprint_way(&[("highway", "footway"), ("surface", "fine_gravel"), extra]);
            assert_eq!(
                found,
                Some(("505.1".to_string(), Target::Candidates)),
                "{extra:?}"
            );
        }
    }

    #[test]
    fn building_passages_and_arcades_are_522_candidates() {
        // ISSprOM 4.5, 522 canopy: "passages, gangways, courts"; ADR 0072. Paved or not, a
        // passage decides routes, so the surface does not matter.
        let passages: [&[(&str, &str)]; 6] = [
            &[("highway", "footway"), ("tunnel", "building_passage")],
            &[
                ("highway", "service"),
                ("tunnel", "building_passage"),
                ("surface", "asphalt"),
            ],
            &[
                ("highway", "footway"),
                ("tunnel", "building_passage"),
                ("surface", "paving_stones"),
            ],
            &[("highway", "footway"), ("covered", "yes")],
            &[("highway", "pedestrian"), ("covered", "arcade")],
            &[
                ("highway", "footway"),
                ("covered", "yes"),
                ("surface", "fine_gravel"),
            ],
        ];
        for tags in passages {
            let found = Rules::embedded().symbol_for(&self::tags(tags), &way(), MapKind::Sprint);
            let found = found.unwrap_or_else(|| panic!("{tags:?}"));
            assert_eq!(found.code, "522", "{tags:?}");
            assert_eq!(found.target, Target::Candidates, "{tags:?}");
            assert!(matches!(found.draw, Draw::Corridor(_)), "{tags:?}");
        }
        // A real tunnel, an indoor way and a ring are no passage.
        for extra in [("tunnel", "yes"), ("indoor", "yes"), ("area", "yes")] {
            let found = sprint_way(&[("highway", "footway"), ("covered", "yes"), extra]);
            assert_ne!(found.map(|f| f.0), Some("522".to_string()), "{extra:?}");
        }
        // A water tunnel under a building is no passage.
        assert_eq!(
            sprint_way(&[("waterway", "stream"), ("tunnel", "building_passage")]),
            None
        );
    }

    #[test]
    fn passage_corridor_has_the_width_of_the_way() {
        let passage = |extra: &[(&str, &str)]| {
            let mut t = vec![("highway", "footway"), ("tunnel", "building_passage")];
            t.extend_from_slice(extra);
            // 6.8 m long, ends cut straight: the area is length times width.
            let way = line("way/9", &t, vec![at(0.0, 0.0), at(0.0, 0.0001)]);
            let (found, set) = objects(&[way], MapKind::Sprint);
            assert_eq!(found.len(), 1);
            assert_eq!(Some(found[0].symbol_id), set.id_for_code("522"));
            let last = found[0].coords.last().unwrap();
            assert_eq!(last.flags, FLAG_HOLE | FLAG_CLOSE);
            let c = &found[0].coords;
            let twice: f64 = (0..c.len())
                .map(|i| {
                    let (a, b) = (&c[i], &c[(i + 1) % c.len()]);
                    f64::from(a.x) * f64::from(b.y) - f64::from(b.x) * f64::from(a.y)
                })
                .sum();
            // Map units are 1/1000 mm; the fixture scale is 1:1000, so 1 m is 1e3 units.
            twice.abs() / 2.0 / 1e6
        };
        let default = passage(&[]);
        assert!((default - 6.8 * 3.0).abs() < 2.0, "{default}");
        let wide = passage(&[("width", "5")]);
        assert!((wide - 6.8 * 5.0).abs() < 2.5, "{wide}");
        // A width below the minimum of a 520/522 stays at the minimum.
        let tiny = passage(&[("width", "0.2")]);
        assert!(tiny > 6.8 * 1.0, "{tiny}");
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
        for condition in ["width<=1.5", "width>2", "width>=", "width>=wide"] {
            let toml =
                format!("[[rule]]\nwhen = [\"{condition}\"]\ndraw = \"line\"\nsprint = \"506\"");
            assert!(Rules::from_toml(&toml).is_err(), "{condition}");
        }
        // `key>=number` compares, like `key<number`.
        let toml = "[[rule]]\nwhen = [\"width>=2\"]\ndraw = \"line\"\nsprint = \"506\"";
        let rules = Rules::from_toml(toml).unwrap();
        let line = OsmGeometry::Line(vec![
            LatLon {
                lat: 52.5,
                lon: 13.4,
            },
            LatLon {
                lat: 52.5,
                lon: 13.401,
            },
        ]);
        let width = |w: &str| vec![("width".to_string(), w.to_string())];
        assert!(
            rules
                .symbol_for(&width("2"), &line, MapKind::Sprint)
                .is_some()
        );
        assert!(
            rules
                .symbol_for(&width("1.9"), &line, MapKind::Sprint)
                .is_none()
        );
    }

    #[test]
    fn value_lists_with_greater_equal_stay_string_matches() {
        // `name=a>=b` is the value `a>=b`, not a comparison.
        let toml = "[[rule]]\nwhen = [\"name=a>=b\"]\ndraw = \"line\"\nsprint = \"506\"";
        let rules = Rules::from_toml(toml).unwrap();
        let line = OsmGeometry::Line(vec![
            LatLon {
                lat: 52.5,
                lon: 13.4,
            },
            LatLon {
                lat: 52.5,
                lon: 13.401,
            },
        ]);
        let name = |v: &str| vec![("name".to_string(), v.to_string())];
        assert!(
            rules
                .symbol_for(&name("a>=b"), &line, MapKind::Sprint)
                .is_some()
        );
        assert!(
            rules
                .symbol_for(&name("9"), &line, MapKind::Sprint)
                .is_none()
        );
        // A value list as the key of a comparison is an error.
        let toml = "[[rule]]\nwhen = [\"a|b>=2\"]\ndraw = \"line\"\nsprint = \"506\"";
        assert!(Rules::from_toml(toml).is_err());
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
        for rule in Rules::embedded()
            .0
            .iter()
            .chain(&Rules::berlin().0)
            .chain(&Rules::states().0)
        {
            for (code, kind) in [
                (&rule.schoolyard, MapKind::Schoolyard),
                (&rule.sprint, MapKind::Sprint),
            ] {
                let Some(code) = code else { continue };
                let set = kind.symbol_set();
                let symbol_type = set.symbol_type(set.id_for_code(code).unwrap());
                let fits: &[u32] = match rule.draw {
                    DrawKind::Point => &[1],
                    DrawKind::Line if rule.corridor_m.is_some() => &[4, 16],
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
    fn relation_area_keeps_its_hole() {
        let d = 0.0002;
        let hole = vec![
            at(0.25 * d, 0.25 * d),
            at(0.25 * d, 0.75 * d),
            at(0.75 * d, 0.75 * d),
            at(0.75 * d, 0.25 * d),
            at(0.25 * d, 0.25 * d),
        ];
        let pitch = OsmFeature {
            id: "relation/7".to_string(),
            tags: tags(&[("leisure", "pitch"), ("surface", "tartan")]),
            geometry: OsmGeometry::Area(vec![OsmPolygon {
                outer: square(),
                holes: vec![hole],
            }]),
        };
        let (objects, set) = objects(&[pitch], MapKind::Schoolyard);
        assert_eq!(objects.len(), 1);
        assert_eq!(Some(objects[0].symbol_id), set.id_for_code("501.10"));
        assert_eq!(objects[0].coords.len(), 10);
        assert_eq!(objects[0].coords[4].flags, FLAG_HOLE | FLAG_CLOSE);
        assert_eq!(objects[0].coords[9].flags, FLAG_HOLE | FLAG_CLOSE);
        let polygon = crate::build::berlin::object_polygon(&objects[0]).unwrap();
        assert_eq!(polygon.interiors().len(), 1);
        assert!(
            objects[0]
                .tags
                .contains(&("freemapper:source_id".to_string(), "relation/7".to_string()))
        );
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
    fn osm_tree_crown_is_tagged() {
        let crown = |tags: &[(&str, &str)]| {
            let (objects, _) = objects(&[point("node/5", tags)], MapKind::Sprint);
            objects[0]
                .tags
                .iter()
                .find(|(k, _)| k == "freemapper:crown_m")
                .map(|(_, v)| v.clone())
        };
        let tree = ("natural", "tree");
        assert_eq!(
            crown(&[tree, ("diameter_crown", "7.5")]).as_deref(),
            Some("7.5")
        );
        assert_eq!(
            crown(&[tree, ("diameter_crown", "6 m")]).as_deref(),
            Some("6")
        );
        assert_eq!(crown(&[tree]), None);
        assert_eq!(crown(&[tree, ("diameter_crown", "large")]), None);
        // An OSM feature cannot set the freemapper tags.
        assert_eq!(crown(&[tree, ("freemapper:crown_m", "20")]), None);
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
