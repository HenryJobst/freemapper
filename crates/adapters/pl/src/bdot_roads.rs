// SPDX-License-Identifier: AGPL-3.0-or-later

//! The roads and paths of BDOT10k (ADR 0187) as a set of extra layers that brings its rules: the
//! road axes (`OT_SKDR_L`) and the footways and cycle ways (`OT_SKRP_L`), for a forest map. The
//! objects rank below OpenStreetMap ([`SetRank::Landscape`]): under `balanced` they are
//! candidates where OpenStreetMap has no path within 10 m, and the engine never draws one twice.
//!
//! The archive of a powiat is 7 to 89 MB, but the two files this set reads are 2.0 MB packed
//! together at powiat 0805 (53.9 MB archive). The set reads them as single members by HTTP byte
//! ranges (ADR 0017), so it needs no cache and downloads about a thirtieth of the archive.
//!
//! The classes in the GML are `ot:OT_SKDR_L` and `ot:OT_SKRP_L`, one `gml:Curve` of
//! `gml:LineStringSegment`s in `ot:geometria`, `posList` of **E N** pairs in EPSG:2180. Left out
//! are the objects that are not in use (`kategoriaIstnienia` other than `eksploatowany`) and the
//! stretches under the ground. The carriageway lines `OT_SKJZ_L` repeat the axes and are not read;
//! the railway lines `OT_SKTR_L` are tracks of trains (OpenStreetMap has them) and are not read.

use std::collections::BTreeMap;
use std::io::BufReader;

use freemapper_adapter_api::alkis::FetchArea;
use freemapper_adapter_api::error::FetchError;
use freemapper_adapter_api::extras::{
    ExtraEstimate, ExtraLayerInfo, ExtraNotes, ExtraSet, ExtraSetInfo, FetchedExtra, SetDrawing,
    SetRank,
};
use freemapper_adapter_api::fetch::{Fetcher, get_with_retries};
use freemapper_contract::MapKind;
use freemapper_georef::crs::{BoundingBox, Crs};
use freemapper_sources::zip_range::{find_member, read_member_up_to, read_members};
use quick_xml::events::Event;
use serde_json::{Value, json};

use crate::bdot10k::{MAX_PACKAGES, Package, index_url, read_index};
use crate::egib::pl1992_inverse;
use crate::gugik_index::{entity, http_error, local};
use crate::rasters::area_in_pl1992;

/// The key of the set: its layers lie in `data/bdot10k` of a project, and its objects carry the
/// source `bdot10k/<layer>`.
pub const KEY: &str = "bdot10k";

/// The layers and the BDOT10k class each is read from, in the order they are fetched.
pub const CLASSES: [(&str, &str); 2] = [("bdot-drogi", "SKDR_L"), ("bdot-sciezki", "SKRP_L")];

/// The properties that stay on a feature (the rules name them as `bdot:<property>`).
const KEPT: [&str; 4] = ["klasaDrogi", "materialNawierzchni", "rodzaj", "szerokosc"];

/// One line object of a class file.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub id: String,
    pub fields: BTreeMap<String, String>,
    /// The points in EPSG:2180, `(east, north)`.
    pub points: Vec<(f64, f64)>,
}

impl Line {
    fn field(&self, name: &str) -> &str {
        self.fields.get(name).map_or("", String::as_str)
    }

    /// Whether the line is on or above the ground and in use.
    fn is_open(&self) -> bool {
        let existing = matches!(self.field("kategoriaIstnienia"), "" | "eksploatowany");
        existing && self.field("polozenie") != "pod powierzchnią gruntu"
    }

    fn touches(&self, bbox: &BoundingBox) -> bool {
        let (mut min_e, mut max_e) = (f64::MAX, f64::MIN);
        let (mut min_n, mut max_n) = (f64::MAX, f64::MIN);
        for &(e, n) in &self.points {
            min_e = min_e.min(e);
            max_e = max_e.max(e);
            min_n = min_n.min(n);
            max_n = max_n.max(n);
        }
        !self.points.is_empty()
            && max_e >= bbox.min_e
            && min_e <= bbox.max_e
            && max_n >= bbox.min_n
            && min_n <= bbox.max_n
    }
}

/// The properties of an object that are read besides the geometry.
const FIELDS: [&str; 8] = [
    "lokalnyId",
    "kategoriaIstnienia",
    "polozenie",
    "klasaDrogi",
    "materialNawierzchni",
    "rodzaj",
    "szerokoscNawierzchni",
    "szerokosc",
];

fn read_pairs(text: &str, source: &str) -> Result<Vec<(f64, f64)>, FetchError> {
    let numbers = text
        .split_whitespace()
        .map(|n| n.parse::<f64>().ok().filter(|v| v.is_finite()))
        .collect::<Option<Vec<_>>>()
        .filter(|n| n.len() % 2 == 0)
        .ok_or_else(|| http_error(source, "a coordinate list is not pairs of numbers"))?;
    Ok(numbers
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| (p[0], p[1]))
        .collect())
}

/// The objects of the class file `class` (`SKDR_L`) that are in use and touch `bbox`
/// (EPSG:2180), streamed. `source` names the file in errors. A file that is no GML, a coordinate
/// list that is no pairs of numbers and a file that ends before its collection closes are errors.
pub fn read_lines(
    input: impl std::io::BufRead,
    class: &str,
    bbox: &BoundingBox,
    source: &str,
) -> Result<Vec<Line>, FetchError> {
    let element = format!("OT_{class}");
    let mut reader = quick_xml::Reader::from_reader(input);
    let mut buffer = Vec::new();
    let (mut depth, mut closed) = (0_usize, false);
    let mut text = String::new();
    let mut draft: Option<Line> = None;
    let mut lines = Vec::new();
    loop {
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(|e| http_error(source, format!("the file is no XML: {e}")))?;
        match event {
            Event::Start(e) => {
                let name = local(e.name().as_ref());
                if depth == 0 {
                    if name != "FeatureCollection" {
                        return Err(http_error(
                            source,
                            format!("the file is no feature collection but <{name}>"),
                        ));
                    }
                } else if name == element {
                    draft = Some(Line {
                        id: String::new(),
                        fields: BTreeMap::new(),
                        points: Vec::new(),
                    });
                }
                depth += 1;
                text.clear();
            }
            Event::Text(t) => text.push_str(t.as_ref()),
            Event::GeneralRef(r) => {
                if let Some(c) = entity(r.as_ref()) {
                    text.push(c);
                }
            }
            Event::End(e) => {
                depth = depth.saturating_sub(1);
                let name = local(e.name().as_ref());
                if let Some(d) = draft.as_mut() {
                    if FIELDS.contains(&name.as_str()) {
                        d.fields.insert(name.clone(), text.trim().to_string());
                    }
                    if name == "posList" {
                        // The segments of a curve follow each other; the joint is listed twice.
                        for point in read_pairs(&text, source)? {
                            if d.points.last() != Some(&point) {
                                d.points.push(point);
                            }
                        }
                    } else if name == element
                        && let Some(mut done) = draft.take()
                    {
                        done.id = done.field("lokalnyId").to_string();
                        if done.is_open() && !done.id.is_empty() && done.touches(bbox) {
                            lines.push(done);
                        }
                    }
                }
                if depth == 0 {
                    closed = true;
                }
                text.clear();
            }
            Event::Empty(e) if depth == 0 => {
                let name = local(e.name().as_ref());
                if name != "FeatureCollection" {
                    return Err(http_error(
                        source,
                        format!("the file is no feature collection but <{name}>"),
                    ));
                }
                closed = true;
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if !closed {
        return Err(http_error(
            source,
            "the file ends before its feature collection closes",
        ));
    }
    Ok(lines)
}

/// The width of the surface in metres, where the register gives one.
fn width(line: &Line) -> Option<f64> {
    ["szerokoscNawierzchni", "szerokosc"]
        .iter()
        .filter_map(|name| line.field(name).parse::<f64>().ok())
        .find(|w| w.is_finite() && *w > 0.0)
}

/// The lines as GeoJSON features in `crs`; a line with a point that has no position or with
/// fewer than two points is left out.
pub fn features(lines: &[Line], crs: Crs) -> Vec<Value> {
    lines
        .iter()
        .filter_map(|line| {
            let coordinates = line
                .points
                .iter()
                .map(|&(east, north)| {
                    let p = crs.forward(pl1992_inverse(east, north)?);
                    Some([p.easting, p.northing])
                })
                .collect::<Option<Vec<_>>>()?;
            if coordinates.len() < 2 {
                return None;
            }
            let mut properties = serde_json::Map::new();
            for name in KEPT {
                if name == "szerokosc" {
                    if let Some(w) = width(line) {
                        properties.insert(name.into(), json!(w));
                    }
                } else if !line.field(name).is_empty() {
                    properties.insert(name.into(), json!(line.field(name)));
                }
            }
            Some(json!({
                "type": "Feature",
                "id": line.id,
                "geometry": {"type": "LineString", "coordinates": coordinates},
                "properties": properties,
            }))
        })
        .collect()
}

/// The limit for one class file, packed and unpacked. The road axes of powiat 0208 (kłodzki, a
/// mountain district) are 76.5 MB unpacked and 8.3 MB packed, above the 64 MB of the shared reader.
const MAX_CLASS_BYTES: u64 = 256 << 20;

/// The lines of `class` in the archive of `package`, read as one member by byte ranges.
fn read_class(
    fetcher: &dyn Fetcher,
    package: &Package,
    class: &str,
    bbox: &BoundingBox,
) -> Result<Vec<Line>, FetchError> {
    let members = read_members(fetcher, &package.url)?;
    let suffix = format!("__ot_{}.xml", class.to_lowercase());
    // A class without an object is a file of 454 bytes, which an archive may also leave out.
    let Some(member) = find_member(&members, |name| name.ends_with(&suffix)) else {
        return Ok(Vec::new());
    };
    let bytes = read_member_up_to(fetcher, &package.url, member, MAX_CLASS_BYTES)?;
    read_lines(BufReader::new(bytes.as_slice()), class, bbox, &package.url)
}

/// The roads and paths of BDOT10k as the set of extra layers of the Polish adapter.
pub struct Roads;

static INFO: ExtraSetInfo = ExtraSetInfo {
    key: KEY,
    crs: Crs::Utm33,
    legacy_field: None,
    notes: ExtraNotes {
        none_stored: "progress-bdot-roads-none-stored",
        removing: "progress-bdot-roads-removing",
        download: "progress-bdot-roads-download",
        features: "progress-bdot-roads-features",
        kept: "warning-bdot-roads-layer-kept",
        dropped: "warning-bdot-roads-layer-skipped",
    },
    // Powiat 0805 (22 November 2026): 2.0 MB for the two class files of the whole powiat, which a
    // map of a few km² downloads once; counted per km² as a map of one km² has them.
    estimate: ExtraEstimate {
        what: "BDOT10k roads and paths (forest maps)",
        megabytes_per_km2: 2.0,
    },
};

static DRAWING: SetDrawing = SetDrawing {
    rules: include_str!("../rules/bdot-roads.toml"),
    tag_prefix: "bdot",
    source_prefix: KEY,
    id_prefix: KEY,
    rank: SetRank::Landscape,
    layer_ranks: &[],
    credited_layers: &[],
    point_dedups: &[],
    kept_tags: &[],
    control_kinds: &[],
    fountain_tags: &[],
    winning_areas: &[],
    line_middles: &[],
    notch_free_layers: &[],
    canopy_yielding: &[],
    canopy_areas: &[],
    cadastre_stairs: &[],
    step_edges: &[],
    facades: &[],
    survey_stairs: &[],
    paving_cuts: &[],
    surveyed_paving: &[],
    carriageway_tags: &[],
    building_parts: None,
    footprint_dates: None,
    stair_layers: &[],
    candidate_layers: &[],
    feature_tags: None,
    generic: true,
};

/// Only a forest map takes the set: a sprint or schoolyard map has the paved areas of the
/// cadastre, and its paths are those of the orthophoto and of OpenStreetMap.
fn takes(kind: MapKind) -> bool {
    kind == MapKind::Forest
}

impl ExtraSet for Roads {
    fn info(&self) -> &ExtraSetInfo {
        &INFO
    }

    fn layers(&self, kind: MapKind) -> Vec<ExtraLayerInfo> {
        if !takes(kind) {
            return Vec::new();
        }
        CLASSES
            .iter()
            .map(|(key, _)| ExtraLayerInfo { key })
            .collect()
    }

    fn has(&self, key: &str) -> bool {
        CLASSES.iter().any(|(layer, _)| *layer == key)
    }

    fn fetch(
        &self,
        fetcher: &dyn Fetcher,
        region_bbox: &BoundingBox,
        kind: MapKind,
    ) -> Vec<FetchedExtra> {
        if !takes(kind) {
            return Vec::new();
        }
        let fail_all = |error: FetchError| -> Vec<FetchedExtra> {
            CLASSES
                .iter()
                .map(|(key, _)| FetchedExtra {
                    key,
                    features: Err(clone_error(&error)),
                })
                .collect()
        };
        let area = FetchArea::new(*region_bbox, Crs::Utm33);
        let Some(bbox) = area_in_pl1992(area.crs, &area.bbox) else {
            return fail_all(http_error(
                crate::bdot10k::INDEX_ENDPOINT,
                "the map area has no position in the Polish grid",
            ));
        };
        let url = index_url(&bbox);
        let packages = match get_with_retries(fetcher, &url, |body| read_index(body, &url)) {
            Ok(packages) if packages.len() > MAX_PACKAGES => {
                return fail_all(http_error(
                    &url,
                    format!("the box touches {} powiats", packages.len()),
                ));
            }
            Ok(packages) => packages,
            Err(error) => return fail_all(error),
        };
        CLASSES
            .iter()
            .map(|(key, class)| {
                let lines = packages.iter().try_fold(Vec::new(), |mut all, package| {
                    all.extend(read_class(fetcher, package, class, &bbox)?);
                    Ok::<_, FetchError>(all)
                });
                FetchedExtra {
                    key,
                    features: lines.map(|lines| features(&lines, Crs::Utm33)),
                }
            })
            .collect()
    }

    fn drawing(&self) -> Option<&SetDrawing> {
        Some(&DRAWING)
    }
}

/// A fetch error repeated for each layer it fails.
fn clone_error(error: &FetchError) -> FetchError {
    http_error("BDOT10k", error.to_string())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use freemapper_adapter_api::testing::MapFetcher;
    use freemapper_sources::zip_range::test_support::archive;

    use super::*;
    use crate::bdot10k::INDEX_ENDPOINT;

    const SKDR: &str = include_str!("../tests/fixtures/bdot10k/SKDR_L.xml");
    const SKRP: &str = include_str!("../tests/fixtures/bdot10k/SKRP_L.xml");
    const INDEX: &str = include_str!("../tests/fixtures/bdot10k/powiaty-slubice.xml");
    const ARCHIVE_URL: &str =
        "https://opendata.geoportal.gov.pl/bdot10k/schemat2021/08/0805_GML.zip";

    /// The box of the fixtures around Słubice, EPSG:2180.
    fn box_2180() -> BoundingBox {
        BoundingBox {
            min_e: 190_000.0,
            min_n: 495_000.0,
            max_e: 205_000.0,
            max_n: 515_000.0,
        }
    }

    fn lines(class: &str, text: &str) -> Vec<Line> {
        read_lines(Cursor::new(text.as_bytes()), class, &box_2180(), "test").unwrap()
    }

    /// A one-object file of `class` with the fields given.
    fn one(class: &str, fields: &str, points: &str) -> String {
        format!(
            "<gml:FeatureCollection xmlns:gml=\"http://www.opengis.net/gml/3.2\" \
             xmlns:ot=\"urn:x\"><gml:featureMember><ot:OT_{class}>\
             <ot:lokalnyId>A1</ot:lokalnyId>{fields}<ot:geometria><gml:Curve><gml:segments>\
             <gml:LineStringSegment><gml:posList>{points}</gml:posList></gml:LineStringSegment>\
             </gml:segments></gml:Curve></ot:geometria></ot:OT_{class}></gml:featureMember>\
             </gml:FeatureCollection>"
        )
    }

    #[test]
    fn the_road_axes_of_a_file_are_read_east_first() {
        let roads = lines("SKDR_L", SKDR);
        assert_eq!(roads.len(), 14);
        let first = &roads[0];
        assert!(!first.id.is_empty());
        // The pairs are E N: Słubice lies near E 198 000, N 507 000, not the other way round.
        for road in &roads {
            for &(e, n) in &road.points {
                assert!((190_000.0..205_000.0).contains(&e), "{e}");
                assert!((495_000.0..515_000.0).contains(&n), "{n}");
            }
        }
        let classes: Vec<&str> = roads.iter().map(|r| r.field("klasaDrogi")).collect();
        assert!(classes.contains(&"droga lokalna"));
        assert!(classes.contains(&"droga główna ruchu przyśpieszonego"));
    }

    #[test]
    fn the_footways_of_a_file_are_read_with_their_kind_and_width() {
        let ways = lines("SKRP_L", SKRP);
        assert_eq!(ways.len(), 9);
        let kinds: Vec<&str> = ways.iter().map(|w| w.field("rodzaj")).collect();
        assert!(kinds.contains(&"ścieżka"));
        assert!(kinds.contains(&"droga dla rowerów"));
        assert!(ways.iter().any(|w| width(w).is_some()));
    }

    #[test]
    fn an_object_not_in_use_or_under_the_ground_is_left_out() {
        let kept = one(
            "SKDR_L",
            "<ot:kategoriaIstnienia>eksploatowany</ot:kategoriaIstnienia>\
             <ot:polozenie>ponad powierzchnią gruntu poziom 1</ot:polozenie>",
            "198000 507000 198100 507000",
        );
        assert_eq!(lines("SKDR_L", &kept).len(), 1, "a bridge stays");
        for fields in [
            "<ot:kategoriaIstnienia>w budowie</ot:kategoriaIstnienia>",
            "<ot:kategoriaIstnienia>nieczynny</ot:kategoriaIstnienia>",
            "<ot:polozenie>pod powierzchnią gruntu</ot:polozenie>",
        ] {
            let gone = one("SKDR_L", fields, "198000 507000 198100 507000");
            assert!(lines("SKDR_L", &gone).is_empty(), "{fields}");
        }
    }

    #[test]
    fn a_line_beside_the_box_is_left_out_and_one_across_it_is_kept() {
        let beside = one("SKDR_L", "", "100000 100000 100100 100000");
        assert!(lines("SKDR_L", &beside).is_empty());
        let across = one("SKDR_L", "", "180000 507000 210000 507000");
        assert_eq!(lines("SKDR_L", &across).len(), 1);
    }

    #[test]
    fn the_segments_of_a_curve_are_one_line_without_a_doubled_joint() {
        let text = "<gml:FeatureCollection xmlns:gml=\"http://www.opengis.net/gml/3.2\" \
            xmlns:ot=\"urn:x\"><gml:featureMember><ot:OT_SKDR_L><ot:lokalnyId>B</ot:lokalnyId>\
            <ot:geometria><gml:Curve><gml:segments>\
            <gml:LineStringSegment><gml:posList>198000 507000 198010 507000</gml:posList>\
            </gml:LineStringSegment><gml:LineStringSegment>\
            <gml:posList>198010 507000 198020 507010</gml:posList></gml:LineStringSegment>\
            </gml:segments></gml:Curve></ot:geometria></ot:OT_SKDR_L></gml:featureMember>\
            </gml:FeatureCollection>";
        let line = &lines("SKDR_L", text)[0];
        assert_eq!(line.points.len(), 3);
    }

    #[test]
    fn a_broken_file_is_an_error_and_an_empty_one_is_empty() {
        let broken = one("SKDR_L", "", "198000 507000 198100");
        let error = read_lines(Cursor::new(broken.as_bytes()), "SKDR_L", &box_2180(), "f");
        assert!(error.is_err());
        let cut = "<gml:FeatureCollection xmlns:gml=\"http://www.opengis.net/gml/3.2\">";
        assert!(read_lines(Cursor::new(cut.as_bytes()), "SKDR_L", &box_2180(), "f").is_err());
        let empty = "<gml:FeatureCollection xmlns:gml=\"http://www.opengis.net/gml/3.2\"/>";
        assert!(
            read_lines(Cursor::new(empty.as_bytes()), "SKDR_L", &box_2180(), "f")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn the_width_is_the_surface_width_of_a_road_or_the_width_of_a_footway() {
        let road = &lines(
            "SKDR_L",
            &one(
                "SKDR_L",
                "<ot:szerokoscNawierzchni uom=\"m\">6.0</ot:szerokoscNawierzchni>",
                "198000 507000 198100 507000",
            ),
        )[0];
        assert_eq!(width(road), Some(6.0));
        let none = &lines("SKDR_L", &one("SKDR_L", "", "198000 507000 198100 507000"))[0];
        assert_eq!(width(none), None);
    }

    #[test]
    fn the_features_are_lines_in_the_map_crs_with_the_properties_the_rules_read() {
        let roads = lines("SKDR_L", SKDR);
        let features = features(&roads, Crs::Utm33);
        assert_eq!(features.len(), roads.len());
        let f = &features[0];
        assert_eq!(f["geometry"]["type"], "LineString");
        // Słubice, 14.6 E 52.35 N: about E 472 000, N 5 800 000 in EPSG:25833.
        let [e, n] = [
            f["geometry"]["coordinates"][0][0].as_f64().unwrap(),
            f["geometry"]["coordinates"][0][1].as_f64().unwrap(),
        ];
        assert!((440_000.0..520_000.0).contains(&e), "{e}");
        assert!((5_750_000.0..5_850_000.0).contains(&n), "{n}");
        assert!(f["properties"]["klasaDrogi"].is_string());
        assert!(f["properties"]["materialNawierzchni"].is_string());
        // Only the properties the rules read stay.
        for (key, _) in f["properties"].as_object().unwrap() {
            assert!(KEPT.contains(&key.as_str()), "{key}");
        }
        let wide = features
            .iter()
            .find(|f| f["properties"]["szerokosc"].is_number());
        assert!(wide.is_some(), "a surveyed width stays a number");
    }

    /// The region box (EPSG:25833) around the fixtures.
    fn region() -> BoundingBox {
        let corner = |east, north| {
            let p = Crs::Utm33.forward(pl1992_inverse(east, north).unwrap());
            (p.easting, p.northing)
        };
        let (a, b) = (corner(190_000.0, 495_000.0), corner(205_000.0, 515_000.0));
        BoundingBox {
            min_e: a.0.min(b.0) - 3_000.0,
            min_n: a.1.min(b.1) - 3_000.0,
            max_e: a.0.max(b.0) + 3_000.0,
            max_n: a.1.max(b.1) + 3_000.0,
        }
    }

    fn served() -> MapFetcher {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(INDEX_ENDPOINT, INDEX.as_bytes().to_vec());
        fetcher.insert(
            ARCHIVE_URL,
            archive(&[
                (
                    "PL.PZGiK.333.0805/BDOT10k/PL.PZGiK.333.0805__OT_SKDR_L.xml",
                    SKDR.as_bytes().to_vec(),
                    true,
                ),
                (
                    "PL.PZGiK.333.0805/BDOT10k/PL.PZGiK.333.0805__OT_SKRP_L.xml",
                    SKRP.as_bytes().to_vec(),
                    true,
                ),
                (
                    "PL.PZGiK.333.0805/BDOT10k/PL.PZGiK.333.0805__OT_BUBD_A.xml",
                    b"not read".to_vec(),
                    true,
                ),
            ]),
        );
        fetcher
    }

    #[test]
    fn a_forest_map_gets_both_layers_from_two_members_of_the_archive() {
        let fetched = Roads.fetch(&served(), &region(), MapKind::Forest);
        let counts: Vec<(&str, Option<usize>)> = fetched
            .iter()
            .map(|f| (f.key, f.features.as_ref().ok().map(Vec::len)))
            .collect();
        assert_eq!(
            counts,
            [("bdot-drogi", Some(14)), ("bdot-sciezki", Some(9))]
        );
    }

    #[test]
    fn the_other_map_kinds_take_nothing_and_ask_nothing() {
        let fetcher = served();
        for kind in [MapKind::Sprint, MapKind::Schoolyard] {
            assert!(Roads.layers(kind).is_empty());
            assert!(Roads.fetch(&fetcher, &region(), kind).is_empty());
        }
        assert_eq!(fetcher.calls.get(), 0);
        assert_eq!(Roads.layers(MapKind::Forest).len(), 2);
        assert!(Roads.has("bdot-drogi") && !Roads.has("bdot-other"));
    }

    #[test]
    fn a_class_the_archive_lacks_is_a_layer_without_objects() {
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(INDEX_ENDPOINT, INDEX.as_bytes().to_vec());
        fetcher.insert(
            ARCHIVE_URL,
            archive(&[("0805/PL__OT_BUBD_A.xml", b"x".to_vec(), true)]),
        );
        let fetched = Roads.fetch(&fetcher, &region(), MapKind::Forest);
        assert!(
            fetched
                .iter()
                .all(|f| f.features.as_ref().is_ok_and(Vec::is_empty))
        );
    }

    #[test]
    fn an_index_that_is_down_fails_both_layers_and_a_broken_member_one() {
        let down = MapFetcher::default();
        down.failing.borrow_mut().push(INDEX_ENDPOINT.to_string());
        let fetched = Roads.fetch(&down, &region(), MapKind::Forest);
        assert_eq!(fetched.len(), 2);
        assert!(fetched.iter().all(|f| f.features.is_err()));

        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(INDEX_ENDPOINT, INDEX.as_bytes().to_vec());
        fetcher.insert(
            ARCHIVE_URL,
            archive(&[
                ("0805/PL__OT_SKDR_L.xml", b"<broken".to_vec(), true),
                ("0805/PL__OT_SKRP_L.xml", SKRP.as_bytes().to_vec(), true),
            ]),
        );
        let fetched = Roads.fetch(&fetcher, &region(), MapKind::Forest);
        assert!(fetched[0].features.is_err(), "the broken class fails");
        assert_eq!(fetched[1].features.as_ref().map(Vec::len).ok(), Some(9));
    }

    #[test]
    fn the_hosts_of_the_roads_are_hosts_the_adapter_already_asks() {
        assert!(crate::bdot10k::INDEX_ENDPOINT.contains("mapy.geoportal.gov.pl"));
        assert!(ARCHIVE_URL.contains(crate::gugik_index::DOWNLOAD_HOST));
    }
}
