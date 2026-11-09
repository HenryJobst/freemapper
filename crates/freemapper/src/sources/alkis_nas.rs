//! The full ALKIS from the states' NAS WFS (NAS-konform, GML 3.2 only, AdV namespace `adv:`):
//! surveyed walls, fences, hedges, embankment edges, bridges and monuments for candidates.
//!
//! One service row per state ([`NAS_SERVICES`]) and one layer row per AdV feature type
//! ([`NAS_LAYERS`]); the AdV schema and its code lists are the same nationwide, so a later state
//! adds a service row only. The services do not page: one GetFeature per type with
//! `COUNT=max_count`, and an answer cut at the cap is asked again in four quadrants.

use std::collections::HashSet;
use std::path::Path;

use serde_json::{Map, Value};

use crate::crs::{BoundingBox, Crs};
use crate::fetch::{
    FetchError, Fetcher, get_limited_with_retries, is_ogc_report, ogc_exception, one_line,
};
use crate::project::MapKind;
use crate::sources::alkis::{AlkisError, FetchArea, invalid, reproject_feature};
use crate::sources::alkis_wfs::resolve_entity;
use crate::sources::region::{Region, regions_for};

/// A NAS WFS of one state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NasService {
    /// For messages, e.g. "LGB ALKIS (NAS)".
    pub name: &'static str,
    pub url: &'static str,
    /// The state the service covers; it is asked only for areas that touch it.
    pub region: Region,
    /// The CRS asked for and answered in.
    pub crs: Crs,
    /// Features per GetFeature (`COUNT`); the service has no paging.
    pub max_count: usize,
    /// The largest answer read; a larger one is stopped and asked again in quadrants.
    pub max_bytes: u64,
    /// The files of the [`NAS_LAYERS`] the service serves; `None`: all of them. A type the state
    /// does not serve answers an exception report, which would warn on every fetch.
    pub layers: Option<&'static [&'static str]>,
    /// The `NAMESPACES` parameter, for a server that wants the `adv` prefix declared.
    pub namespaces: Option<&'static str>,
}

/// 256 MB: 100 000 features of about 1.1 KB (Sanssouci) and room to spare.
const MAX_ANSWER_BYTES: u64 = 256 << 20;

/// LGB, Brandenburg: "WFS BB ALKIS NAS-konform" (GID 7.1, EPSG:25833; `CountDefault` 100 000,
/// `ImplementsResultPaging` FALSE).
pub const LGB_NAS: NasService = NasService {
    name: "LGB ALKIS (NAS)",
    url: "https://isk.geobasis-bb.de/ows/alkis_nas_wfs",
    region: Region::Brandenburg,
    crs: Crs::Utm33,
    max_count: 100_000,
    max_bytes: MAX_ANSWER_BYTES,
    layers: None,
    namespaces: None,
};

/// The NAS services, one per state.
pub const NAS_SERVICES: [NasService; 1] = [LGB_NAS];

/// One AdV feature type, stored as `data/alkis-nas/<file>.geojson` for the map kinds that use it.
#[derive(Debug, PartialEq)]
pub struct NasLayer {
    pub file: &'static str,
    /// `adv:AX_…` as the services know it.
    pub type_name: &'static str,
    pub kinds: &'static [MapKind],
}

const BOTH_KINDS: &[MapKind] = &[MapKind::Sprint, MapKind::Schoolyard];
const SPRINT_ONLY: &[MapKind] = &[MapKind::Sprint];

/// The NAS layers in fetch order. `AX_BoeschungKliff` has no geometry property (the services
/// answer an exception), and `AX_Bauteil` is not fetched: its parts are the vereinfacht building
/// parts already stored (vereinfacht `oid` = NAS `gml:id` + `BL`).
pub const NAS_LAYERS: [NasLayer; 6] = [
    NasLayer {
        file: "sonstiges-bauwerk",
        type_name: "adv:AX_SonstigesBauwerkOderSonstigeEinrichtung",
        kinds: BOTH_KINDS,
    },
    NasLayer {
        file: "vegetationsmerkmal",
        type_name: "adv:AX_Vegetationsmerkmal",
        kinds: BOTH_KINDS,
    },
    NasLayer {
        file: "strukturlinie",
        type_name: "adv:AX_Strukturlinie3D",
        kinds: SPRINT_ONLY,
    },
    NasLayer {
        file: "bauwerk-verkehr",
        type_name: "adv:AX_BauwerkImVerkehrsbereich",
        kinds: SPRINT_ONLY,
    },
    NasLayer {
        file: "bauwerk-gewaesser",
        type_name: "adv:AX_BauwerkImGewaesserbereich",
        kinds: BOTH_KINDS,
    },
    NasLayer {
        file: "untergeordnetes-gewaesser",
        type_name: "adv:AX_UntergeordnetesGewaesser",
        kinds: BOTH_KINDS,
    },
];

/// How often an area is split into quadrants when an answer is cut at the cap: 4 levels, down
/// to 1/256 of the area.
pub const NAS_SPLIT_MAX_DEPTH: u32 = 4;

/// The largest angle one straight piece of a densified arc spans.
pub const ARC_STEP_DEG: f64 = 10.0;

/// The NAS layers `kind` uses.
pub fn nas_layers(kind: MapKind) -> impl Iterator<Item = &'static NasLayer> {
    NAS_LAYERS.iter().filter(move |l| l.kinds.contains(&kind))
}

/// The NAS services of the states `region_bbox` (EPSG:25833) touches.
pub fn nas_services_for(region_bbox: &BoundingBox) -> Vec<&'static NasService> {
    let regions = regions_for(region_bbox);
    NAS_SERVICES
        .iter()
        .filter(|s| regions.contains(&s.region))
        .collect()
}

impl NasService {
    /// Whether the service serves `layer`.
    pub fn serves(&self, layer: &NasLayer) -> bool {
        self.layers.is_none_or(|files| files.contains(&layer.file))
    }

    /// The GetFeature of `type_name` features in `bbox` (in [`NasService::crs`]), rounded
    /// outward, with `COUNT` = [`NasService::max_count`].
    pub fn get_feature_url(&self, type_name: &str, bbox: &BoundingBox) -> String {
        let epsg = self.crs.epsg();
        format!(
            "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={type_name}\
             &SRSNAME=urn:ogc:def:crs:EPSG::{epsg}&BBOX={},{},{},{},urn:ogc:def:crs:EPSG::{epsg}\
             &COUNT={}{}",
            self.url,
            bbox.min_e.floor(),
            bbox.min_n.floor(),
            bbox.max_e.ceil(),
            bbox.max_n.ceil(),
            self.max_count,
            self.namespaces
                .map(|n| format!("&NAMESPACES={n}"))
                .unwrap_or_default()
        )
    }
}

/// One NAS answer: its features as GeoJSON (in the service's CRS), the features it could not
/// read (`<gml:id> (<type>): <why>`), and its `numberMatched` (`None`: "unknown") and
/// `numberReturned` (the members counted when the attribute is missing).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NasAnswer {
    pub features: Vec<Value>,
    pub skipped: Vec<String>,
    pub matched: Option<usize>,
    pub returned: usize,
}

/// The features of a NAS `wfs:FeatureCollection`. Each GeoJSON feature has `id` = `gml:id` and
/// the properties `oid` (= `gml:id`), `nas_type` (the AdV feature type, e.g.
/// `AX_Vegetationsmerkmal`), every simple property as text (codes such as `bauwerksfunktion`
/// `1740`; a property with one nested value, such as `objekthoehe`, gets that value), `beginnt`
/// (the date of `lebenszeitintervall/beginnt`) and `gehoert_zu` (the oid an `xlink:href` of a
/// `gehoertZu…` property points to). An answer that is not a collection (an exception report,
/// an HTML page) is an error; a feature whose geometry it cannot read is skipped and listed.
pub fn parse_nas_gml(text: &str) -> Result<NasAnswer, AlkisError> {
    use quick_xml::events::Event;
    let refused = || {
        if is_ogc_report(text.as_bytes()) {
            let detail = ogc_exception(text.as_bytes()).unwrap_or_default();
            return invalid(format!("exception report: {detail}"));
        }
        invalid(format!(
            "not a WFS FeatureCollection: {}",
            one_line(text.trim_start(), 80)
        ))
    };
    let mut reader = quick_xml::Reader::from_str(text);
    let mut answer = NasAnswer::default();
    let mut members = 0;
    let mut returned = None;
    // Local names of the open elements below the collection's root: `member`, the feature,
    // its property, and what the property holds.
    let mut path: Vec<String> = Vec::new();
    let mut root_seen = false;
    let mut feature: Option<NasFeature> = None;
    let mut text_buf = String::new();
    loop {
        let event = reader
            .read_event()
            .map_err(|e| invalid(format!("GML: {e}")))?;
        let empty = matches!(event, Event::Empty(_));
        match event {
            Event::Start(e) | Event::Empty(e) => {
                let name = local(e.name().as_ref());
                // By local name: `gml:id` and `xlink:href` whatever the prefixes are.
                let attribute = |key: &str| {
                    e.attributes()
                        .flatten()
                        .find(|a| local(a.key.as_ref()) == key)
                        .map(|a| a.value.to_string())
                };
                if !root_seen {
                    if name != "FeatureCollection" {
                        return Err(refused());
                    }
                    root_seen = true;
                    answer.matched = attribute("numberMatched").and_then(|m| m.parse().ok());
                    returned = attribute("numberReturned").and_then(|r| r.parse().ok());
                    if empty {
                        break;
                    }
                    continue;
                }
                let depth = path.len();
                match (depth, feature.as_mut()) {
                    // wfs:member / <AX_…>; wfs:boundedBy holds no feature.
                    (1, None) if path[0] == "member" => {
                        members += 1;
                        let started =
                            NasFeature::new(attribute("id").unwrap_or_default(), name.clone());
                        if empty {
                            // A feature without any property.
                            answer.skipped.extend(started.into_geojson().err());
                        } else {
                            feature = Some(started);
                        }
                    }
                    (2, Some(f)) => {
                        f.leaves = 0;
                        f.leaf.clear();
                        if name == "position" {
                            f.has_position = true;
                        }
                        if name.starts_with("gehoertZu")
                            && let Some(href) = attribute("href")
                        {
                            let oid = href.strip_prefix(OID_URN).unwrap_or(&href);
                            f.properties
                                .entry(RELATION_KEY)
                                .or_insert_with(|| Value::from(oid));
                        }
                    }
                    (3.., Some(f)) if path[2] == "position" => {
                        f.geometry.open(&name, depth, &attribute);
                        if empty {
                            f.geometry.close(&name, depth, "");
                        }
                    }
                    _ => {}
                }
                text_buf.clear();
                if !empty {
                    path.push(name);
                }
            }
            Event::CData(t) => text_buf.push_str(&t.into_inner()),
            Event::Text(t) => text_buf.push_str(t.as_ref()),
            Event::GeneralRef(r) => text_buf.extend(resolve_entity(r.as_ref())),
            Event::End(_) => {
                let Some(name) = path.pop() else { continue };
                let depth = path.len();
                if let Some(f) = feature.as_mut() {
                    let value = text_buf.trim();
                    match depth {
                        1 => {
                            if let Some(done) = feature.take() {
                                match done.into_geojson() {
                                    Ok(geojson) => answer.features.push(geojson),
                                    Err(why) => answer.skipped.push(why),
                                }
                            }
                        }
                        3.. if path[2] == "position" => f.geometry.close(&name, depth, value),
                        _ => {
                            if name == "beginnt" {
                                let date: String = value.chars().take(10).collect();
                                f.properties.insert("beginnt".into(), date.into());
                            } else if !value.is_empty() {
                                f.leaves += 1;
                                f.leaf = value.to_string();
                            }
                            // A property with one value, directly or nested (`objekthoehe`).
                            if depth == 2
                                && f.leaves == 1
                                && !NOT_PROPERTIES.contains(&name.as_str())
                            {
                                let leaf = std::mem::take(&mut f.leaf);
                                f.properties.entry(name).or_insert_with(|| leaf.into());
                            }
                        }
                    }
                }
                text_buf.clear();
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !root_seen {
        return Err(refused());
    }
    answer.returned = returned.unwrap_or(members);
    Ok(answer)
}

/// The prefix of an AdV object id in an `xlink:href`.
const OID_URN: &str = "urn:adv:oid:";
/// The property that holds the oid a `gehoertZu…` relation points to.
const RELATION_KEY: &str = "gehoert_zu";
/// Direct children of a NAS feature that are not stored as properties: the object id, the
/// version interval (its `beginnt` is), the model, the occasion, the quality notes and the
/// geometry.
const NOT_PROPERTIES: [&str; 7] = [
    "identifier",
    "lebenszeitintervall",
    "modellart",
    "anlass",
    "qualitaetsangaben",
    "position",
    "boundedBy",
];

/// The GML elements of `position` the reader knows; anything else (clothoids, splines,
/// B-splines, …) is not read, and the feature is listed as skipped instead.
const GEOMETRY_ELEMENTS: [&str; 27] = [
    "Point",
    "pos",
    "posList",
    "LineString",
    "Curve",
    "CompositeCurve",
    "MultiCurve",
    "OrientableCurve",
    "baseCurve",
    "curveMember",
    "segments",
    "LineStringSegment",
    "Arc",
    "ArcString",
    "Circle",
    "Surface",
    "CompositeSurface",
    "MultiSurface",
    "surfaceMember",
    "patches",
    "PolygonPatch",
    "Polygon",
    "exterior",
    "interior",
    "Ring",
    "LinearRing",
    "curveMembers",
];

/// One feature while it is read.
struct NasFeature {
    id: String,
    nas_type: String,
    properties: Map<String, Value>,
    /// The values seen in the open property so far, and the last of them.
    leaves: usize,
    leaf: String,
    has_position: bool,
    geometry: GeometryBuilder,
}

impl NasFeature {
    fn new(id: String, nas_type: String) -> Self {
        let mut properties = Map::new();
        properties.insert("oid".into(), id.clone().into());
        properties.insert("nas_type".into(), nas_type.clone().into());
        Self {
            id,
            nas_type,
            properties,
            leaves: 0,
            leaf: String::new(),
            has_position: false,
            geometry: GeometryBuilder::default(),
        }
    }

    /// The GeoJSON feature, or why it was skipped (`<gml:id> (<type>): <why>`).
    fn into_geojson(self) -> Result<Value, String> {
        let why = |why: &str| format!("{} ({}): {why}", self.id, self.nas_type);
        if !self.has_position {
            return Err(why("no geometry"));
        }
        let geometry = self.geometry.finish().map_err(|e| why(&e))?;
        Ok(serde_json::json!({
            "type": "Feature",
            "id": self.id,
            "geometry": geometry,
            "properties": self.properties,
        }))
    }
}

type Point = [f64; 2];

/// The geometry of one `position` while it is read: curve pieces (segments in reading order),
/// the polygons built from closed rings, or a point.
#[derive(Default)]
struct GeometryBuilder {
    /// Coordinates of the open coordinate list(s) of a segment, ring or point.
    coords: Vec<Point>,
    /// Curve pieces not yet in a ring.
    pieces: Vec<Vec<Point>>,
    /// Where the pieces of each open ring start.
    rings: Vec<usize>,
    /// Open `OrientableCurve`s: where their pieces start and whether they run backwards.
    orientations: Vec<(usize, bool)>,
    /// `srsDimension` of open elements, by path depth.
    dimensions: Vec<(usize, usize)>,
    polygons: Vec<Vec<Vec<Point>>>,
    point: Option<Point>,
    /// Why the geometry cannot be read.
    error: Option<String>,
}

impl GeometryBuilder {
    fn fail(&mut self, why: String) {
        self.error.get_or_insert(why);
    }

    fn open(&mut self, name: &str, depth: usize, attribute: &dyn Fn(&str) -> Option<String>) {
        if !GEOMETRY_ELEMENTS.contains(&name) {
            self.fail(format!("unsupported GML element {name}"));
            return;
        }
        if let Some(dimension) = attribute("srsDimension") {
            match dimension.parse::<usize>() {
                Ok(d) if d >= 2 => self.dimensions.push((depth, d)),
                _ => self.fail(format!("srsDimension {}", one_line(&dimension, 20))),
            }
        }
        match name {
            "Polygon" | "PolygonPatch" => self.polygons.push(Vec::new()),
            "Ring" | "LinearRing" => self.rings.push(self.pieces.len()),
            "OrientableCurve" => {
                let backwards = attribute("orientation").is_some_and(|o| o == "-");
                self.orientations.push((self.pieces.len(), backwards));
            }
            _ => {}
        }
    }

    fn close(&mut self, name: &str, depth: usize, text: &str) {
        if self.error.is_some() {
            return;
        }
        match name {
            "posList" | "pos" => {
                let dimension = self.dimensions.last().map_or(2, |&(_, d)| d);
                let numbers: Result<Vec<f64>, _> =
                    text.split_ascii_whitespace().map(str::parse).collect();
                match numbers {
                    Ok(numbers) if numbers.len().is_multiple_of(dimension) => self
                        .coords
                        .extend(numbers.chunks_exact(dimension).map(|c| [c[0], c[1]])),
                    _ => self.fail(format!("unreadable {name}")),
                }
            }
            "LineStringSegment" | "LineString" | "LinearRing" => {
                let piece = std::mem::take(&mut self.coords);
                self.pieces.push(piece);
            }
            "Arc" | "ArcString" | "Circle" => {
                let given = std::mem::take(&mut self.coords);
                match densify_arcs(&given, name == "Circle") {
                    Some(piece) => self.pieces.push(piece),
                    None => self.fail(format!("{name} of {} points", given.len())),
                }
            }
            "Point" => self.point = std::mem::take(&mut self.coords).first().copied(),
            "OrientableCurve" => {
                if let Some((start, true)) = self.orientations.pop() {
                    self.pieces[start..].reverse();
                    self.pieces[start..].iter_mut().for_each(|p| p.reverse());
                }
            }
            _ => {}
        }
        if matches!(name, "Ring" | "LinearRing") {
            self.close_ring();
        }
        self.dimensions.retain(|&(d, _)| d < depth);
    }

    /// The pieces of the open ring as one closed ring of the last polygon.
    fn close_ring(&mut self) {
        let start = self.rings.pop().unwrap_or(0);
        let mut ring = join(self.pieces.drain(start..));
        if ring.first() != ring.last() {
            ring.push(ring[0]);
        }
        if ring.len() < 4 {
            self.fail(format!("ring of {} points", ring.len()));
            return;
        }
        match self.polygons.last_mut() {
            Some(polygon) => polygon.push(ring),
            None => self.polygons.push(vec![ring]),
        }
    }

    /// The GeoJSON geometry: a Point, a (Multi)Polygon of the rings, or a (Multi)LineString of
    /// the pieces (pieces that touch end to start make one line).
    fn finish(mut self) -> Result<Value, String> {
        if let Some(error) = self.error {
            return Err(error);
        }
        if let Some([e, n]) = self.point {
            return Ok(serde_json::json!({ "type": "Point", "coordinates": [e, n] }));
        }
        self.polygons.retain(|p| !p.is_empty());
        match self.polygons.len() {
            0 => {}
            1 => {
                let polygon = self.polygons.pop();
                return Ok(serde_json::json!({ "type": "Polygon", "coordinates": polygon }));
            }
            _ => {
                return Ok(
                    serde_json::json!({ "type": "MultiPolygon", "coordinates": self.polygons }),
                );
            }
        }
        let mut lines: Vec<Vec<Point>> = Vec::new();
        for piece in self.pieces.into_iter().map(|p| join([p])) {
            match lines.last_mut() {
                Some(line) if line.last() == piece.first() => line.extend(&piece[1..]),
                _ => lines.push(piece),
            }
        }
        lines.retain(|l| l.len() >= 2);
        match lines.len() {
            0 => Err("no geometry".to_string()),
            1 => Ok(serde_json::json!({ "type": "LineString", "coordinates": lines[0] })),
            _ => Ok(serde_json::json!({ "type": "MultiLineString", "coordinates": lines })),
        }
    }
}

/// `pieces` end to end as one point list, without the doubled point where one ends and the next
/// starts (or any other point repeated in a row).
fn join(pieces: impl IntoIterator<Item = Vec<Point>>) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::new();
    for p in pieces.into_iter().flatten() {
        if out.last() != Some(&p) {
            out.push(p);
        }
    }
    out
}

/// The arcs through `given` (start, middle, end; an `ArcString` chains them, each end the next
/// start) as one line with pieces of at most [`ARC_STEP_DEG`], from the start to the end through
/// the middle points; `full` (`gml:Circle`): the whole circle through the three points, back to
/// the first. Three points on a line are a straight line. `None` for a wrong number of points.
fn densify_arcs(given: &[Point], full: bool) -> Option<Vec<Point>> {
    if full && given.len() != 3 || given.len() < 3 || given.len().is_multiple_of(2) {
        return None;
    }
    let mut out = vec![given[0]];
    for triple in given.windows(3).step_by(2) {
        out.extend(arc(triple[0], triple[1], triple[2], full));
    }
    Some(out)
}

/// The points of the arc from `a` through `b` to `c` after `a`, ending exactly on `c` (on `a`
/// when `full`).
fn arc(a: Point, b: Point, c: Point, full: bool) -> Vec<Point> {
    use std::f64::consts::TAU;
    // Relative to `a`: UTM coordinates squared lose precision.
    let (bx, by) = (b[0] - a[0], b[1] - a[1]);
    let (cx, cy) = (c[0] - a[0], c[1] - a[1]);
    let cross = bx * cy - by * cx;
    let scale = (bx * bx + by * by).max(cx * cx + cy * cy);
    if cross.abs() <= 1e-12 * scale {
        return if full { vec![b, a] } else { vec![b, c] };
    }
    let (b2, c2) = (bx * bx + by * by, cx * cx + cy * cy);
    let ux = (cy * b2 - by * c2) / (2.0 * cross);
    let uy = (bx * c2 - cx * b2) / (2.0 * cross);
    let radius = ux.hypot(uy);
    let start = (-uy).atan2(-ux);
    // A positive cross product turns counter-clockwise from `a` over `b` to `c`.
    let sweep = if full {
        TAU.copysign(cross)
    } else {
        let ccw = ((cy - uy).atan2(cx - ux) - start).rem_euclid(TAU);
        if cross > 0.0 { ccw } else { ccw - TAU }
    };
    let steps = (sweep.abs() / ARC_STEP_DEG.to_radians() - 1e-9)
        .ceil()
        .max(1.0) as usize;
    let mut points: Vec<Point> = (1..steps)
        .map(|i| {
            let angle = start + sweep * i as f64 / steps as f64;
            [
                a[0] + ux + radius * angle.cos(),
                a[1] + uy + radius * angle.sin(),
            ]
        })
        .collect();
    points.push(if full { a } else { c });
    points
}

/// The element name without its namespace prefix.
fn local(name: &str) -> String {
    name.rsplit(':').next().unwrap_or_default().to_string()
}

/// The features of one NAS layer, from every service of the area, in the map's CRS.
#[derive(Debug, Clone, PartialEq)]
pub struct NasFeatures {
    pub features: Vec<Value>,
    /// Features no geometry could be read for, as [`NasAnswer::skipped`].
    pub skipped: Vec<String>,
    /// The states whose services were asked.
    pub regions: Vec<Region>,
}

/// One NAS layer as fetched; an empty `features` is a layer without objects in the area.
#[derive(Debug)]
pub struct FetchedNasLayer {
    pub layer: &'static NasLayer,
    pub features: Result<NasFeatures, FetchError>,
}

/// The NAS layers of `kind` for `area`; none at all, and no request, when the area touches no
/// state with a NAS service. A layer that fails does not stop the others; one service that fails
/// fails the layer (a half-filled file would pass for complete). Features two services or two
/// quadrants both return (same `gml:id`) are kept once. Each answer streams into the file `part`
/// first (removed afterwards), so an oversized one is stopped while it is read.
pub fn fetch_nas_layers(
    fetcher: &dyn Fetcher,
    area: &FetchArea,
    kind: MapKind,
    part: &Path,
) -> Vec<FetchedNasLayer> {
    fetch_layers_from(
        fetcher,
        &nas_services_for(&area.region_bbox),
        area,
        kind,
        part,
    )
}

/// [`fetch_nas_layers`] from `services`: the layers of `kind` that at least one of them serves,
/// each from the services that serve it.
fn fetch_layers_from(
    fetcher: &dyn Fetcher,
    services: &[&NasService],
    area: &FetchArea,
    kind: MapKind,
    part: &Path,
) -> Vec<FetchedNasLayer> {
    nas_layers(kind)
        .filter_map(|layer| {
            let serving: Vec<&NasService> = services
                .iter()
                .copied()
                .filter(|s| s.serves(layer))
                .collect();
            (!serving.is_empty()).then(|| FetchedNasLayer {
                layer,
                features: fetch_layer(fetcher, &serving, layer, area, part),
            })
        })
        .collect()
}

/// `layer` from each of `services`, reprojected into the map's CRS.
fn fetch_layer(
    fetcher: &dyn Fetcher,
    services: &[&NasService],
    layer: &NasLayer,
    area: &FetchArea,
    part: &Path,
) -> Result<NasFeatures, FetchError> {
    let mut seen = HashSet::new();
    let mut fetched = NasFeatures {
        features: Vec::new(),
        skipped: Vec::new(),
        regions: Vec::new(),
    };
    for service in services {
        let request = Request {
            fetcher,
            service,
            type_name: layer.type_name,
            part,
        };
        let mut answer = NasAnswer::default();
        let bbox = area.bbox_for(service.crs);
        fetch_split(&request, &bbox, 0, &mut seen, &mut answer)?;
        fetched.features.extend(
            answer
                .features
                .into_iter()
                .map(|f| reproject_feature(f, service.crs, area.crs)),
        );
        fetched.skipped.extend(answer.skipped);
        fetched.regions.push(service.region);
    }
    Ok(fetched)
}

/// All `type_name` features of `service` in `bbox` (in the service's CRS) and the skipped ones.
/// An answer cut at the cap (`numberReturned < numberMatched`, or `numberMatched` unknown and
/// `numberReturned >= max_count`) is asked again in four quadrants, at most
/// [`NAS_SPLIT_MAX_DEPTH`] levels deep, and so is an answer larger than `max_bytes`. Answers
/// stream into the file `part`.
pub fn fetch_nas_type(
    fetcher: &dyn Fetcher,
    service: &NasService,
    type_name: &str,
    bbox: &BoundingBox,
    part: &Path,
) -> Result<NasAnswer, FetchError> {
    let request = Request {
        fetcher,
        service,
        type_name,
        part,
    };
    let mut answer = NasAnswer::default();
    fetch_split(&request, bbox, 0, &mut HashSet::new(), &mut answer)?;
    answer.matched = Some(answer.features.len() + answer.skipped.len());
    answer.returned = answer.features.len() + answer.skipped.len();
    Ok(answer)
}

/// What one type is asked for, and where its answers stream to.
struct Request<'a> {
    fetcher: &'a dyn Fetcher,
    service: &'a NasService,
    type_name: &'a str,
    part: &'a Path,
}

/// [`fetch_nas_type`] for `bbox` at split level `depth`, adding to `out` the features whose
/// `gml:id` is not in `seen` yet.
fn fetch_split(
    request: &Request,
    bbox: &BoundingBox,
    depth: u32,
    seen: &mut HashSet<String>,
    out: &mut NasAnswer,
) -> Result<(), FetchError> {
    let service = request.service;
    let answer = match get_answer(request, bbox) {
        Err(FetchError::TooLarge { .. }) if depth < NAS_SPLIT_MAX_DEPTH => {
            for quadrant in quadrants(bbox) {
                fetch_split(request, &quadrant, depth + 1, seen, out)?;
            }
            return Ok(());
        }
        answer => answer?,
    };
    let cut = match answer.matched {
        Some(matched) => answer.returned < matched,
        None => answer.returned >= service.max_count,
    };
    if cut {
        if depth >= NAS_SPLIT_MAX_DEPTH {
            let of = answer
                .matched
                .map_or_else(|| "an unknown number of".to_string(), |m| m.to_string());
            return Err(FetchError::Alkis(invalid(format!(
                "{} returned {} of {of} features; choose a smaller area",
                service.name, answer.returned
            ))));
        }
        for quadrant in quadrants(bbox) {
            fetch_split(request, &quadrant, depth + 1, seen, out)?;
        }
        return Ok(());
    }
    for feature in answer.features {
        let id = feature["id"].as_str().unwrap_or_default();
        if id.is_empty() || seen.insert(id.to_string()) {
            out.features.push(feature);
        }
    }
    for skipped in answer.skipped {
        if !out.skipped.contains(&skipped) {
            out.skipped.push(skipped);
        }
    }
    Ok(())
}

/// One GetFeature in `bbox`, read: retried like every download, but an exception report (the type
/// has no geometry, the request is refused) and an answer over `max_bytes` are not asked again.
fn get_answer(request: &Request, bbox: &BoundingBox) -> Result<NasAnswer, FetchError> {
    let Request {
        fetcher,
        service,
        type_name,
        part,
    } = *request;
    let url = service.get_feature_url(type_name, bbox);
    let name = service.name;
    get_limited_with_retries(fetcher, &url, part, service.max_bytes, |bytes| {
        if is_ogc_report(bytes) {
            return Err(FetchError::Refused {
                url: url.clone(),
                status: 200,
                detail: ogc_exception(bytes),
            });
        }
        parse_nas_gml(&String::from_utf8_lossy(bytes))
            .map_err(|e| FetchError::Alkis(invalid(format!("{url}: {}", e.message))))
    })
    .map_err(|e| match e {
        FetchError::Refused { status, detail, .. } => {
            let detail = detail.map(|d| format!(": {d}")).unwrap_or_default();
            let what = if status == 200 {
                "an exception report".to_string()
            } else {
                format!("a refusal (HTTP {status})")
            };
            FetchError::Alkis(invalid(format!(
                "{name} answered {type_name} with {what}{detail}"
            )))
        }
        e => e,
    })
}

/// The four quarters of `bbox`.
fn quadrants(bbox: &BoundingBox) -> [BoundingBox; 4] {
    let mid_e = (bbox.min_e + bbox.max_e) / 2.0;
    let mid_n = (bbox.min_n + bbox.max_n) / 2.0;
    let part = |min_e, min_n, max_e, max_n| BoundingBox {
        min_e,
        min_n,
        max_e,
        max_n,
    };
    [
        part(bbox.min_e, bbox.min_n, mid_e, mid_n),
        part(mid_e, bbox.min_n, bbox.max_e, mid_n),
        part(bbox.min_e, mid_n, mid_e, bbox.max_n),
        part(mid_e, mid_n, bbox.max_e, bbox.max_n),
    ]
}

#[cfg(test)]
pub(crate) mod test_support {
    pub const SONSTIGES_BAUWERK: &str =
        include_str!("../../../../tests/fixtures/brandenburg-nas-sonstiges-bauwerk.gml");
    pub const VEGETATION: &str =
        include_str!("../../../../tests/fixtures/brandenburg-nas-vegetation.gml");
    pub const STRUKTURLINIE: &str =
        include_str!("../../../../tests/fixtures/brandenburg-nas-strukturlinie.gml");

    /// A NAS answer with `numberMatched` = `matched` and one line feature per id.
    pub fn answer(matched: &str, ids: &[&str]) -> String {
        let members: String = ids
            .iter()
            .map(|id| {
                format!(
                    r#"<wfs:member><AX_SonstigesBauwerkOderSonstigeEinrichtung gml:id="{id}"><position><gml:LineString><gml:posList>366300 5807200 366310 5807210</gml:posList></gml:LineString></position><bauwerksfunktion>1740</bauwerksfunktion></AX_SonstigesBauwerkOderSonstigeEinrichtung></wfs:member>"#
                )
            })
            .collect();
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?><wfs:FeatureCollection numberReturned="{}" numberMatched="{matched}" xmlns="http://www.adv-online.de/namespaces/adv/gid/7.1" xmlns:gml="http://www.opengis.net/gml/3.2" xmlns:wfs="http://www.opengis.net/wfs/2.0" xmlns:xlink="http://www.w3.org/1999/xlink">{members}</wfs:FeatureCollection>"#,
            ids.len()
        )
    }

    /// The empty answer of a layer without objects in the area (a self-closing collection).
    pub const EMPTY: &str = r#"<?xml version="1.0" encoding="utf-8"?><wfs:FeatureCollection numberReturned="0" numberMatched="0" xmlns:gml="http://www.opengis.net/gml/3.2" xmlns:wfs="http://www.opengis.net/wfs/2.0"/>"#;

    /// An exception report as the service sends it for a type without geometry.
    pub const EXCEPTION: &str = r#"<?xml version="1.0" encoding="UTF-8"?><ows:ExceptionReport xmlns:ows="http://www.opengis.net/ows/1.1" version="2.0.0"><ows:Exception exceptionCode="InvalidParameterValue" locator="typeNames"><ows:ExceptionText>no geometry property</ows:ExceptionText></ows:Exception></ows:ExceptionReport>"#;
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;
    use crate::fetch::test_support::MapFetcher;

    /// Sanssouci, Potsdam (EPSG:25833), where the fixtures come from.
    const SANSSOUCI: BoundingBox = BoundingBox {
        min_e: 366_200.0,
        min_n: 5_807_100.0,
        max_e: 367_100.0,
        max_n: 5_807_700.0,
    };

    /// A collection with the members `members` (GML text).
    fn collection(members: &str) -> String {
        format!(
            r#"<wfs:FeatureCollection numberReturned="1" numberMatched="1" xmlns="http://www.adv-online.de/namespaces/adv/gid/7.1" xmlns:gml="http://www.opengis.net/gml/3.2" xmlns:wfs="http://www.opengis.net/wfs/2.0" xmlns:xlink="http://www.w3.org/1999/xlink">{members}</wfs:FeatureCollection>"#
        )
    }

    /// One `AX_SonstigesBauwerkOderSonstigeEinrichtung` member with `position` = `geometry`.
    fn member(id: &str, geometry: &str) -> String {
        format!(
            r#"<wfs:member><AX_SonstigesBauwerkOderSonstigeEinrichtung gml:id="{id}"><position>{geometry}</position><bauwerksfunktion>1740</bauwerksfunktion></AX_SonstigesBauwerkOderSonstigeEinrichtung></wfs:member>"#
        )
    }

    fn points(geometry: &Value) -> Vec<[f64; 2]> {
        geometry["coordinates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap()])
            .collect()
    }

    fn by_id<'a>(answer: &'a NasAnswer, id: &str) -> &'a Value {
        answer
            .features
            .iter()
            .find(|f| f["id"] == id)
            .unwrap_or_else(|| panic!("{id} in {:?}", answer.skipped))
    }

    #[test]
    fn nas_curve_with_arc_becomes_line() {
        // A half circle of radius 10 through its top: densified in steps of 10°, the right way
        // round (every point above the chord), ending exactly on the last point.
        let (e, n) = (366_400.0, 5_807_400.0);
        let arc = |p: [f64; 6]| {
            format!(
                "<gml:Curve><gml:segments><gml:Arc><gml:posList>{} {} {} {} {} {}</gml:posList></gml:Arc></gml:segments></gml:Curve>",
                e + p[0],
                n + p[1],
                e + p[2],
                n + p[3],
                e + p[4],
                n + p[5]
            )
        };
        let top = member("DEBBAL0000000001", &arc([-10.0, 0.0, 0.0, 10.0, 10.0, 0.0]));
        let bottom = member(
            "DEBBAL0000000002",
            &arc([10.0, 0.0, 0.0, -10.0, -10.0, 0.0]),
        );
        let parsed = parse_nas_gml(&collection(&(top + &bottom))).unwrap();
        for (id, side) in [("DEBBAL0000000001", 1.0), ("DEBBAL0000000002", -1.0)] {
            let line = &by_id(&parsed, id)["geometry"];
            assert_eq!(line["type"], "LineString");
            let line = points(line);
            assert_eq!(line.len(), 19, "180° in 18 pieces: {line:?}");
            assert_eq!(line[0], [e - 10.0 * side, n]);
            assert_eq!(line[18], [e + 10.0 * side, n]);
            for p in &line {
                let r = (p[0] - e).hypot(p[1] - n);
                assert!((r - 10.0).abs() < 1e-6, "{p:?} on the circle");
                assert!(
                    (p[1] - n) * side >= -1e-9,
                    "{p:?} on the side of the middle point"
                );
            }
        }
        // A full circle (`gml:Circle`) closes on its first point.
        let circle = member(
            "DEBBAL0000000003",
            &format!(
                "<gml:Curve><gml:segments><gml:Circle><gml:posList>{} {n} {e} {} {} {n}</gml:posList></gml:Circle></gml:segments></gml:Curve>",
                e - 5.0,
                n + 5.0,
                e + 5.0
            ),
        );
        let parsed = parse_nas_gml(&collection(&circle)).unwrap();
        let ring = points(&by_id(&parsed, "DEBBAL0000000003")["geometry"]);
        assert_eq!(ring.len(), 37);
        assert_eq!(ring[0], ring[36]);
        // A curve used backwards runs from its end to its start.
        let backwards = member(
            "DEBBAL0000000004",
            r#"<gml:CompositeCurve><gml:curveMember><gml:OrientableCurve orientation="-"><gml:baseCurve><gml:Curve><gml:segments><gml:LineStringSegment><gml:posList>0 0 1 0</gml:posList></gml:LineStringSegment><gml:LineStringSegment><gml:posList>1 0 1 1</gml:posList></gml:LineStringSegment></gml:segments></gml:Curve></gml:baseCurve></gml:OrientableCurve></gml:curveMember><gml:curveMember><gml:LineString><gml:posList>0 0 -1 0</gml:posList></gml:LineString></gml:curveMember></gml:CompositeCurve>"#,
        );
        let parsed = parse_nas_gml(&collection(&backwards)).unwrap();
        assert_eq!(
            by_id(&parsed, "DEBBAL0000000004")["geometry"]["coordinates"],
            serde_json::json!([[1.0, 1.0], [1.0, 0.0], [0.0, 0.0], [-1.0, 0.0]])
        );
        // Live hedges: arcs and straight segments of one curve make one line, without doubled
        // points at the joins, longer than the straight posLists alone.
        let parsed = parse_nas_gml(VEGETATION).unwrap();
        assert_eq!(parsed.features.len(), 3, "{:?}", parsed.skipped);
        assert!(parsed.skipped.is_empty());
        for feature in &parsed.features {
            assert_eq!(feature["geometry"]["type"], "LineString", "{feature}");
            let line = points(&feature["geometry"]);
            assert!(line.windows(2).all(|w| w[0] != w[1]), "{line:?}");
        }
        let mixed = points(&by_id(&parsed, "DEBBAL540001vq9T")["geometry"]);
        assert_eq!(mixed[0], [366_727.581, 5_807_585.932]);
        assert_eq!(mixed[0], *mixed.last().unwrap(), "a closed hedge");
        let given = VEGETATION
            .split("gml:id=\"DEBBAL540001vq9T\"")
            .nth(1)
            .unwrap()
            .split("</wfs:member>")
            .next()
            .unwrap()
            .matches("<gml:posList>")
            .count();
        assert!(mixed.len() > given * 2, "{} points", mixed.len());
        // Walls with arcs in a CompositeCurve, too.
        let parsed = parse_nas_gml(SONSTIGES_BAUWERK).unwrap();
        let wall = points(&by_id(&parsed, "DEBBAL540001vq3A")["geometry"]);
        assert_eq!(wall[0], [366_546.605, 5_807_566.740]);
        assert_eq!(*wall.last().unwrap(), [366_543.091, 5_807_572.108]);
        assert!(wall.len() > 6);
    }

    #[test]
    fn nas_surface_of_curve_rings_becomes_polygon() {
        // The canopy of the Sanssouci fixture: a ring of two curve members, closed.
        let parsed = parse_nas_gml(SONSTIGES_BAUWERK).unwrap();
        assert_eq!(parsed.features.len(), 5, "{:?}", parsed.skipped);
        let canopy = &by_id(&parsed, "DEBBAL540001vmqB")["geometry"];
        assert_eq!(canopy["type"], "Polygon");
        let rings = canopy["coordinates"].as_array().unwrap();
        assert_eq!(rings.len(), 1);
        let ring = points(&serde_json::json!({ "coordinates": rings[0] }));
        assert_eq!(ring.len(), 7, "{ring:?}");
        assert_eq!(ring[0], [366_452.539, 5_807_339.060]);
        assert_eq!(ring[0], ring[6]);
        // An open ring is closed; a hole stays a hole; two surfaces make a MultiPolygon.
        let open = "<gml:Ring><gml:curveMember><gml:Curve><gml:segments><gml:LineStringSegment><gml:posList>0 0 10 0 10 10 0 10</gml:posList></gml:LineStringSegment></gml:segments></gml:Curve></gml:curveMember></gml:Ring>";
        let hole = "<gml:LinearRing><gml:posList>2 2 4 2 4 4 2 2</gml:posList></gml:LinearRing>";
        let polygon = format!(
            "<gml:Polygon><gml:exterior>{open}</gml:exterior><gml:interior>{hole}</gml:interior></gml:Polygon>"
        );
        let multi = format!(
            "<gml:MultiSurface><gml:surfaceMember>{polygon}</gml:surfaceMember><gml:surfaceMember><gml:Surface><gml:patches><gml:PolygonPatch><gml:exterior>{open}</gml:exterior></gml:PolygonPatch></gml:patches></gml:Surface></gml:surfaceMember></gml:MultiSurface>"
        );
        let parsed = parse_nas_gml(&collection(
            &(member("DEBBAL0000000001", &polygon) + &member("DEBBAL0000000002", &multi)),
        ))
        .unwrap();
        let one = &by_id(&parsed, "DEBBAL0000000001")["geometry"];
        assert_eq!(one["type"], "Polygon");
        assert_eq!(
            one["coordinates"],
            serde_json::json!([
                [
                    [0.0, 0.0],
                    [10.0, 0.0],
                    [10.0, 10.0],
                    [0.0, 10.0],
                    [0.0, 0.0]
                ],
                [[2.0, 2.0], [4.0, 2.0], [4.0, 4.0], [2.0, 2.0]]
            ])
        );
        let two = &by_id(&parsed, "DEBBAL0000000002")["geometry"];
        assert_eq!(two["type"], "MultiPolygon");
        assert_eq!(two["coordinates"].as_array().unwrap().len(), 2);
        // A geometry the reader does not know, and a feature without one, are listed, not an error.
        let unknown = member(
            "DEBBAL0000000003",
            "<gml:Curve><gml:segments><gml:Clothoid><gml:posList>0 0 1 1</gml:posList></gml:Clothoid></gml:segments></gml:Curve>",
        );
        let none = r#"<wfs:member><AX_BoeschungKliff gml:id="DEBBAL0000000004"><beginnt>2020-01-01</beginnt></AX_BoeschungKliff></wfs:member>"#;
        let parsed = parse_nas_gml(&collection(&(unknown + none))).unwrap();
        assert!(parsed.features.is_empty());
        assert_eq!(
            parsed.skipped,
            [
                "DEBBAL0000000003 (AX_SonstigesBauwerkOderSonstigeEinrichtung): unsupported GML element Clothoid",
                "DEBBAL0000000004 (AX_BoeschungKliff): no geometry"
            ]
        );
    }

    #[test]
    fn nas_3d_poslist_drops_height() {
        let line = member(
            "DEBBAL0000000001",
            r#"<gml:LineString srsDimension="3"><gml:posList>1 2 30 3 4 40 5 6 50</gml:posList></gml:LineString>"#,
        );
        let own = member(
            "DEBBAL0000000002",
            r#"<gml:Curve><gml:segments><gml:LineStringSegment><gml:posList srsDimension="3">1 2 30 3 4 40</gml:posList></gml:LineStringSegment></gml:segments></gml:Curve>"#,
        );
        let point = member(
            "DEBBAL0000000003",
            r#"<gml:Point srsDimension="3"><gml:pos>7 8 9</gml:pos></gml:Point>"#,
        );
        let plain = member(
            "DEBBAL0000000004",
            "<gml:Point><gml:pos>7 8</gml:pos></gml:Point>",
        );
        let parsed = parse_nas_gml(&collection(&(line + &own + &point + &plain))).unwrap();
        assert!(parsed.skipped.is_empty(), "{:?}", parsed.skipped);
        assert_eq!(
            by_id(&parsed, "DEBBAL0000000001")["geometry"]["coordinates"],
            serde_json::json!([[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]])
        );
        assert_eq!(
            by_id(&parsed, "DEBBAL0000000002")["geometry"]["coordinates"],
            serde_json::json!([[1.0, 2.0], [3.0, 4.0]])
        );
        for id in ["DEBBAL0000000003", "DEBBAL0000000004"] {
            assert_eq!(
                by_id(&parsed, id)["geometry"],
                serde_json::json!({ "type": "Point", "coordinates": [7.0, 8.0] })
            );
        }
        // The live 3D structure lines are served in 2D.
        let parsed = parse_nas_gml(STRUKTURLINIE).unwrap();
        assert_eq!(parsed.features.len(), 4, "{:?}", parsed.skipped);
        let first = points(&by_id(&parsed, "DEBBAL540001BNEB")["geometry"]);
        assert_eq!(first[0], [366_402.079, 5_807_381.342]);
        assert_eq!(first.len(), 7);
    }

    #[test]
    fn nas_feature_keeps_codes_date_and_relation() {
        let parsed = parse_nas_gml(SONSTIGES_BAUWERK).unwrap();
        let wall = by_id(&parsed, "DEBBAL540001vq3A");
        assert_eq!(
            wall["properties"],
            serde_json::json!({
                "oid": "DEBBAL540001vq3A",
                "nas_type": "AX_SonstigesBauwerkOderSonstigeEinrichtung",
                "beginnt": "2013-01-29",
                "bauwerksfunktion": "1702",
            })
        );
        let parsed = parse_nas_gml(STRUKTURLINIE).unwrap();
        let top = &by_id(&parsed, "DEBBAL540001BNEB")["properties"];
        assert_eq!(top["art"], "1220");
        assert_eq!(top["nas_type"], "AX_Strukturlinie3D");
        assert!(top.get("qualitaetsangaben").is_none(), "{top}");
        // A stair edge points to its stair (MV shape); a height is one nested value.
        let edge = r#"<wfs:member><AX_SonstigesBauwerkOderSonstigeEinrichtung gml:id="DEMVHR030000xCDq"><lebenszeitintervall><AA_Lebenszeitintervall><beginnt>2019-04-02T10:11:12Z</beginnt></AA_Lebenszeitintervall></lebenszeitintervall><position><gml:LineString><gml:posList>1 2 3 4</gml:posList></gml:LineString></position><bauwerksfunktion>1630</bauwerksfunktion><objekthoehe><AX_RelativeHoehe><hoehe uom="urn:adv:uom:m">3.500</hoehe></AX_RelativeHoehe></objekthoehe><name>Treppe &amp; Rampe</name><gehoertZuBauwerk_AX_SonstigesBauwerkOderSonstigeEinrichtung xlink:href="urn:adv:oid:DEMVHR030000xwpx"/></AX_SonstigesBauwerkOderSonstigeEinrichtung></wfs:member>"#;
        let parsed = parse_nas_gml(&collection(edge)).unwrap();
        let properties = &by_id(&parsed, "DEMVHR030000xCDq")["properties"];
        assert_eq!(properties["gehoert_zu"], "DEMVHR030000xwpx");
        assert_eq!(properties["beginnt"], "2019-04-02");
        assert_eq!(properties["bauwerksfunktion"], "1630");
        assert_eq!(properties["objekthoehe"], "3.500");
        assert_eq!(properties["name"], "Treppe & Rampe");
        assert_eq!((parsed.matched, parsed.returned), (Some(1), 1));
    }

    #[test]
    fn nas_answer_over_the_cap_is_split_into_quadrants() {
        const TYPE: &str = "adv:AX_SonstigesBauwerkOderSonstigeEinrichtung";
        let service = NasService {
            max_count: 3,
            ..LGB_NAS
        };
        let scratch = tempfile::tempdir().unwrap();
        let part = scratch.path().join("answer.gml.part");
        assert_eq!(
            service.get_feature_url(TYPE, &SANSSOUCI),
            "https://isk.geobasis-bb.de/ows/alkis_nas_wfs?SERVICE=WFS&VERSION=2.0.0\
             &REQUEST=GetFeature&TYPENAMES=adv:AX_SonstigesBauwerkOderSonstigeEinrichtung\
             &SRSNAME=urn:ogc:def:crs:EPSG::25833\
             &BBOX=366200,5807100,367100,5807700,urn:ogc:def:crs:EPSG::25833&COUNT=3"
        );
        let quadrant = |min_e: f64, min_n: f64| BoundingBox {
            min_e,
            min_n,
            max_e: min_e + 450.0,
            max_n: min_n + 300.0,
        };
        let quadrants = [
            (
                quadrant(366_200.0, 5_807_100.0),
                vec!["DEBBAL01", "DEBBAL02"],
            ),
            // An object across the quadrant border comes twice.
            (
                quadrant(366_650.0, 5_807_100.0),
                vec!["DEBBAL02", "DEBBAL03"],
            ),
            (quadrant(366_200.0, 5_807_400.0), vec!["DEBBAL04"]),
            (quadrant(366_650.0, 5_807_400.0), vec!["DEBBAL05"]),
        ];
        for matched in ["5", "unknown"] {
            let fetcher = MapFetcher::default();
            let cut = answer(matched, &["DEBBAL01", "DEBBAL02", "DEBBAL03"]);
            fetcher.insert(&service.get_feature_url(TYPE, &SANSSOUCI), cut.into_bytes());
            for (bbox, ids) in &quadrants {
                let part = answer(&ids.len().to_string(), ids);
                fetcher.insert(&service.get_feature_url(TYPE, bbox), part.into_bytes());
            }
            let fetched = fetch_nas_type(&fetcher, &service, TYPE, &SANSSOUCI, &part).unwrap();
            let mut ids: Vec<&str> = fetched
                .features
                .iter()
                .map(|f| f["id"].as_str().unwrap())
                .collect();
            ids.sort_unstable();
            assert_eq!(
                ids,
                ["DEBBAL01", "DEBBAL02", "DEBBAL03", "DEBBAL04", "DEBBAL05"],
                "numberMatched={matched}"
            );
            assert_eq!(fetcher.calls.get(), 5, "the area, then its quadrants");
        }
        // Unknown but below the cap: complete, not split.
        let fetcher = MapFetcher::default();
        let below = answer("unknown", &["DEBBAL01", "DEBBAL02"]);
        fetcher.insert(
            &service.get_feature_url(TYPE, &SANSSOUCI),
            below.into_bytes(),
        );
        let fetched = fetch_nas_type(&fetcher, &service, TYPE, &SANSSOUCI, &part).unwrap();
        assert_eq!(fetched.features.len(), 2);
        assert_eq!(fetcher.calls.get(), 1);
        // Cut on every level: an error after 4 splits.
        let fetcher = MapFetcher::default();
        let cut = answer("5", &["DEBBAL01", "DEBBAL02", "DEBBAL03"]);
        fetcher.insert_prefix(service.url, cut.into_bytes());
        let err = fetch_nas_type(&fetcher, &service, TYPE, &SANSSOUCI, &part).unwrap_err();
        assert_eq!(
            err.to_string(),
            "invalid ALKIS data: LGB ALKIS (NAS) returned 3 of 5 features; choose a smaller area"
        );
        assert_eq!(fetcher.calls.get(), 1 + NAS_SPLIT_MAX_DEPTH as usize);
    }

    #[test]
    fn nas_layers_only_in_nas_states() {
        let files = |kind: MapKind, bbox: BoundingBox| {
            let fetcher = MapFetcher::default();
            fetcher.insert_prefix(LGB_NAS.url, EMPTY.as_bytes().to_vec());
            let scratch = tempfile::tempdir().unwrap();
            let part = scratch.path().join("answer.gml.part");
            let area = FetchArea::new(bbox, Crs::Utm33);
            let fetched = fetch_nas_layers(&fetcher, &area, kind, &part);
            let files: Vec<&str> = fetched
                .iter()
                .map(|l| {
                    let features = l.features.as_ref().unwrap();
                    assert!(features.features.is_empty());
                    assert_eq!(features.regions, [Region::Brandenburg]);
                    l.layer.file
                })
                .collect();
            (files, fetcher.calls.get())
        };
        let strausberg = BoundingBox {
            min_e: 421_000.0,
            min_n: 5_821_000.0,
            max_e: 422_000.0,
            max_n: 5_822_000.0,
        };
        let (sprint, calls) = files(MapKind::Sprint, strausberg);
        assert_eq!(
            sprint,
            [
                "sonstiges-bauwerk",
                "vegetationsmerkmal",
                "strukturlinie",
                "bauwerk-verkehr",
                "bauwerk-gewaesser",
                "untergeordnetes-gewaesser"
            ]
        );
        assert_eq!(calls, 6);
        // Embankment edges and bridges are sprint-only.
        let (schoolyard, calls) = files(MapKind::Schoolyard, strausberg);
        assert_eq!(
            schoolyard,
            [
                "sonstiges-bauwerk",
                "vegetationsmerkmal",
                "bauwerk-gewaesser",
                "untergeordnetes-gewaesser"
            ]
        );
        assert_eq!(calls, 4);
        // Berlin Mitte and Dresden: no NAS service, no request.
        let mitte = BoundingBox {
            min_e: 392_000.0,
            min_n: 5_820_000.0,
            max_e: 393_500.0,
            max_n: 5_821_000.0,
        };
        assert_eq!(regions_for(&mitte), [Region::Berlin]);
        let dresden = BoundingBox {
            min_e: 415_800.0,
            min_n: 5_655_600.0,
            max_e: 416_150.0,
            max_n: 5_655_900.0,
        };
        for bbox in [mitte, dresden] {
            for kind in [MapKind::Sprint, MapKind::Schoolyard] {
                assert_eq!(files(kind, bbox), (Vec::new(), 0));
            }
        }
    }

    #[test]
    fn nas_answer_over_the_size_cap_is_split() {
        const TYPE: &str = "adv:AX_SonstigesBauwerkOderSonstigeEinrichtung";
        let whole = answer("3", &["DEBBAL01", "DEBBAL02", "DEBBAL03"]);
        let half = answer("2", &["DEBBAL01", "DEBBAL02"]);
        // Complete by count, but larger than the service reads at once.
        let service = NasService {
            max_bytes: half.len() as u64,
            ..LGB_NAS
        };
        let scratch = tempfile::tempdir().unwrap();
        let part = scratch.path().join("answer.gml.part");
        let fetcher = MapFetcher::default();
        fetcher.insert(
            &service.get_feature_url(TYPE, &SANSSOUCI),
            whole.clone().into_bytes(),
        );
        let [a, b, c, d] = quadrants(&SANSSOUCI);
        for (bbox, ids) in [
            (a, &["DEBBAL01", "DEBBAL02"][..]),
            (b, &["DEBBAL03"][..]),
            (c, &[][..]),
            (d, &[][..]),
        ] {
            let text = answer(&ids.len().to_string(), ids);
            fetcher.insert(&service.get_feature_url(TYPE, &bbox), text.into_bytes());
        }
        let fetched = fetch_nas_type(&fetcher, &service, TYPE, &SANSSOUCI, &part).unwrap();
        assert_eq!(fetched.features.len(), 3);
        assert_eq!(fetcher.calls.get(), 5, "the area, then its quadrants");
        assert!(!part.exists(), "the part file is removed");
        // Too large on every level: an error after 4 splits, not asked again.
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(service.url, whole.into_bytes());
        let err = fetch_nas_type(&fetcher, &service, TYPE, &SANSSOUCI, &part).unwrap_err();
        assert!(matches!(err, FetchError::TooLarge { .. }), "{err}");
        assert_eq!(fetcher.calls.get(), 1 + NAS_SPLIT_MAX_DEPTH as usize);
    }

    #[test]
    fn nas_service_asks_only_its_layers() {
        // A state whose service lacks most types, and needs the prefix declared.
        static ONLY_WALLS: [&str; 1] = ["sonstiges-bauwerk"];
        let service = NasService {
            layers: Some(&ONLY_WALLS),
            namespaces: Some("xmlns(adv,http://www.adv-online.de/namespaces/adv/gid/7.1)"),
            ..LGB_NAS
        };
        assert!(
            service
                .get_feature_url(NAS_LAYERS[0].type_name, &SANSSOUCI)
                .ends_with("&COUNT=100000&NAMESPACES=xmlns(adv,http://www.adv-online.de/namespaces/adv/gid/7.1)")
        );
        assert!(service.serves(&NAS_LAYERS[0]) && !service.serves(&NAS_LAYERS[1]));
        assert!(LGB_NAS.serves(&NAS_LAYERS[5]), "LGB serves every layer");
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(service.url, EMPTY.as_bytes().to_vec());
        let scratch = tempfile::tempdir().unwrap();
        let part = scratch.path().join("answer.gml.part");
        let area = FetchArea::new(SANSSOUCI, Crs::Utm33);
        let fetched = fetch_layers_from(&fetcher, &[&service], &area, MapKind::Sprint, &part);
        let files: Vec<&str> = fetched.iter().map(|l| l.layer.file).collect();
        assert_eq!(files, ["sonstiges-bauwerk"]);
        assert_eq!(
            fetcher.calls.get(),
            1,
            "no request for a type the service lacks"
        );
    }
}
