// SPDX-License-Identifier: AGPL-3.0-or-later

//! The cadastre of Poland, EGiB (`ewidencja gruntów i budynków`), as GUGiK serves it: the
//! national WFS 2.0.0 `UslugaZbiorcza` with the layers `ms:budynki` (buildings) and `ms:dzialki`
//! (parcels). Every answer is GML 3.2 in EPSG:2180, a polygon as a `posList` of `N E` pairs (the
//! axis order of the CRS), and a page holds at most 1000 features; a full page names the next
//! one in the attribute `next` of the collection (see `docs/regions/poland-reference.md`).
//!
//! The client turns both layers into the LGB schema the engine reads ([`buildings`],
//! [`land_use`]): a building is a `Gebäude` whose `funktion` follows the kind of the register
//! (`RODZAJ`), and a parcel gets the land use `nutzart` its register class (`KLASOUZYTKI_EGIB`)
//! stands for, only where that class is clear. A parcel of two classes would colour a field with
//! the wood beside it, so it gets none.

use std::collections::HashSet;

use freemapper_adapter_api::alkis::{AlkisPart, FetchArea};
use freemapper_adapter_api::error::FetchError;
use freemapper_adapter_api::fetch::{Fetcher, get_with_retries, is_ogc_report};
use freemapper_georef::crs::{BoundingBox, Crs, LatLon, Pl1992};
use quick_xml::events::Event;
use serde_json::{Value, json};

use crate::gugik_index::{attribute, entity, http_error, local, report_error};
use crate::rasters::area_in_pl1992;

/// The WFS of the national cadastre.
pub const ENDPOINT: &str =
    "https://mapy.geoportal.gov.pl/wss/service/PZGIK/EGIB/WFS/UslugaZbiorcza";

/// Features asked for per page: the default and the most the service gives.
pub const PAGE: usize = 1000;

/// Pages read for one area at most. A map area is a few square kilometres, and a page holds a
/// thousand features, so a service that keeps pointing to a next page is stopped here.
const MAX_PAGES: usize = 200;

/// The two layers of the service the client reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Buildings,
    Parcels,
}

impl Layer {
    fn typename(self) -> &'static str {
        match self {
            Layer::Buildings => "ms:budynki",
            Layer::Parcels => "ms:dzialki",
        }
    }

    /// The element of one feature.
    fn element(self) -> &'static str {
        match self {
            Layer::Buildings => "budynki",
            Layer::Parcels => "dzialki",
        }
    }

    /// The attribute that identifies a feature.
    fn id_field(self) -> &'static str {
        match self {
            Layer::Buildings => "ID_BUDYNKU",
            Layer::Parcels => "ID_DZIALKI",
        }
    }

    /// The attribute that says what a feature is.
    fn kind_field(self) -> &'static str {
        match self {
            Layer::Buildings => "RODZAJ",
            Layer::Parcels => "KLASOUZYTKI_EGIB",
        }
    }
}

/// The request of the first page of `layer` in `bbox` (EPSG:2180). The service takes the box
/// with the axes of the CRS, north first.
pub fn first_url(layer: Layer, bbox: &BoundingBox) -> String {
    let crs = "urn:ogc:def:crs:EPSG::2180";
    format!(
        "{ENDPOINT}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={}&COUNT={PAGE}\
         &SRSNAME={crs}&BBOX={},{},{},{},{crs}",
        layer.typename(),
        bbox.min_n.floor(),
        bbox.min_e.floor(),
        bbox.max_n.ceil(),
        bbox.max_e.ceil(),
    )
}

/// A feature as the service sends it: its id, its kind and its polygons in EPSG:2180, each a ring
/// list (the first is the outer ring) of `(north, east)` points.
#[derive(Debug, Clone, PartialEq)]
pub struct Feature {
    pub id: String,
    pub kind: String,
    pub polygons: Vec<Vec<Vec<(f64, f64)>>>,
}

/// One page of an answer.
#[derive(Debug, Default, PartialEq)]
pub struct Page {
    pub features: Vec<Feature>,
    pub next: Option<String>,
}

/// Reads a page. An exception report, an answer that is no feature collection and one that ends
/// early are errors the caller asks again for.
pub fn read_page(layer: Layer, body: &[u8], url: &str) -> Result<Page, FetchError> {
    if is_ogc_report(body) {
        return Err(report_error(url, body));
    }
    let mut reader = quick_xml::Reader::from_reader(body);
    let mut buffer = Vec::new();
    let mut page = Page::default();
    let (mut depth, mut root_closed) = (0_usize, false);
    let mut text = String::new();
    let mut draft: Option<Feature> = None;
    let mut ring: Option<Vec<(f64, f64)>> = None;
    let mut rings: Vec<Vec<(f64, f64)>> = Vec::new();
    loop {
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(|e| http_error(url, format!("the answer is no XML: {e}")))?;
        match event {
            Event::Start(e) => {
                let name = local(e.name().as_ref());
                if depth == 0 {
                    if name != "FeatureCollection" {
                        return Err(http_error(
                            url,
                            format!("the answer is no feature collection but <{name}>"),
                        ));
                    }
                    page.next = attribute(&e, "next");
                } else if draft.is_none() && name == layer.element() {
                    draft = Some(Feature {
                        id: String::new(),
                        kind: String::new(),
                        polygons: Vec::new(),
                    });
                } else if draft.is_some() && name == "Polygon" {
                    rings.clear();
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
                    if name == layer.id_field() {
                        d.id = text.trim().to_string();
                    } else if name == layer.kind_field() {
                        d.kind = text.trim().to_string();
                    } else if name == "posList" {
                        ring = Some(read_ring(&text, url)?);
                    } else if name == "LinearRing" {
                        rings.extend(ring.take());
                    } else if name == "Polygon" && !rings.is_empty() {
                        d.polygons.push(std::mem::take(&mut rings));
                    } else if name == layer.element()
                        && let Some(done) = draft.take()
                    {
                        page.features.push(done);
                    }
                }
                if depth == 0 {
                    root_closed = true;
                }
                text.clear();
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if !root_closed {
        return Err(http_error(
            url,
            "the answer ends before its feature collection closes",
        ));
    }
    Ok(page)
}

/// The points of a `posList`: pairs of `N E`.
fn read_ring(text: &str, url: &str) -> Result<Vec<(f64, f64)>, FetchError> {
    let numbers = text
        .split_whitespace()
        .map(|n| n.parse::<f64>().ok().filter(|v| v.is_finite()))
        .collect::<Option<Vec<_>>>()
        .filter(|n| n.len() % 2 == 0)
        .ok_or_else(|| http_error(url, "a coordinate list is not pairs of numbers"))?;
    Ok(numbers
        .as_chunks::<2>()
        .0
        .iter()
        .map(|p| (p[0], p[1]))
        .collect())
}

/// Every feature of `layer` in `bbox` (EPSG:2180), page by page. A page that is full names the
/// next; a short one ends the read.
pub fn fetch_all(
    fetcher: &dyn Fetcher,
    layer: Layer,
    bbox: &BoundingBox,
) -> Result<Vec<Feature>, FetchError> {
    let mut features = Vec::new();
    let mut seen = HashSet::new();
    let mut visited = HashSet::new();
    let mut url = first_url(layer, bbox);
    for _ in 0..MAX_PAGES {
        visited.insert(url.clone());
        let page = get_with_retries(fetcher, &url, |body| read_page(layer, body, &url))?;
        let full = page.features.len() >= PAGE;
        features.extend(
            page.features
                .into_iter()
                .filter(|f| f.id.is_empty() || seen.insert(f.id.clone())),
        );
        match page.next.filter(|n| full && !visited.contains(n)) {
            Some(next) => url = next,
            None => return Ok(features),
        }
    }
    Err(http_error(
        &url,
        format!("the service names more than {MAX_PAGES} pages"),
    ))
}

/// The position of a point of EPSG:2180. The core projects to PL-1992 and not back, so the
/// position is found by Newton's method on the projection: from a guess on the meridian, with the
/// slopes taken over a step of a microdegree. Five rounds are exact to far below a millimetre
/// anywhere in Poland; `None` when a point does not converge.
pub(crate) fn pl1992_inverse(east: f64, north: f64) -> Option<LatLon> {
    const STEP: f64 = 1e-6;
    let at = |lat: f64, lon: f64| {
        let p = Pl1992::forward(LatLon { lat, lon });
        (p.easting, p.northing)
    };
    let mut lat = 52.0;
    let mut lon = 19.0 + (east - 500_000.0) / 68_500.0;
    for _ in 0..8 {
        let (e0, n0) = at(lat, lon);
        let (de, dn) = (east - e0, north - n0);
        if de.abs() < 1e-6 && dn.abs() < 1e-6 {
            break;
        }
        let (e_lat, n_lat) = at(lat + STEP, lon);
        let (e_lon, n_lon) = at(lat, lon + STEP);
        // The matrix of the slopes per microdegree, solved for the correction.
        let (a, b) = ((e_lat - e0) / STEP, (e_lon - e0) / STEP);
        let (c, d) = ((n_lat - n0) / STEP, (n_lon - n0) / STEP);
        let det = a * d - b * c;
        if det.abs() < f64::EPSILON {
            return None;
        }
        lat += (de * d - b * dn) / det;
        lon += (a * dn - c * de) / det;
    }
    let (e, n) = at(lat, lon);
    ((e - east).abs() < 1e-3 && (n - north).abs() < 1e-3).then_some(LatLon { lat, lon })
}

/// A point of EPSG:2180 in `crs`, which is that of the map.
fn to_map(crs: Crs, (north, east): (f64, f64)) -> Option<[f64; 2]> {
    let ll = pl1992_inverse(east, north)?;
    let p = crs.forward(ll);
    Some([p.easting, p.northing])
}

/// The geometry of a feature as GeoJSON in `crs`; `None` when a point has no position.
fn geometry(feature: &Feature, crs: Crs) -> Option<Value> {
    let polygons = feature
        .polygons
        .iter()
        .map(|rings| {
            rings
                .iter()
                .map(|ring| {
                    ring.iter()
                        .map(|p| to_map(crs, *p))
                        .collect::<Option<Vec<_>>>()
                })
                .collect::<Option<Vec<_>>>()
        })
        .collect::<Option<Vec<_>>>()?;
    Some(json!({"type": "MultiPolygon", "coordinates": polygons}))
}

/// The `funktion` of a building of the register's kind `RODZAJ`. The letters are the kinds of
/// the ordinance on the register: `m` residential, `g` farm and other outbuildings, `p`
/// industrial, `t` transport and communication, `h` commercial and services, `z` tanks and
/// silos, `b` office, `s` hospitals, `o` education, science and culture, `k` sports, `i` other.
/// A kind the table does not know keeps its letter, so the data stays readable.
pub fn building_function(kind: &str) -> String {
    match kind {
        "m" => "Wohngebäude",
        "g" => "Gebäude für Wirtschaft und Gewerbe",
        "p" => "Industriegebäude",
        "t" => "Gebäude für Verkehr und Nachrichtenwesen",
        "h" => "Handels- und Dienstleistungsgebäude",
        "z" => "Behälter, Silo und Speichergebäude",
        "b" => "Bürogebäude",
        "s" => "Gebäude für Gesundheit und Pflege",
        "o" => "Gebäude für Bildung, Wissenschaft und Kultur",
        "k" => "Gebäude für Sport",
        "i" => "Sonstiges Gebäude",
        other => other,
    }
    .to_string()
}

/// The land use of one register class of a parcel, as `(nutzart, bez)` in the LGB spellings the
/// map reads. Only what is clear: a class the table does not know, or one that does not say what
/// lies on the ground (built-up land, waste land), gives `None`.
///
/// `Ls` forest, `Lz` trees and shrubs outside the forest, `Ł` meadow, `Ps` pasture, `R` arable,
/// `S` orchard, `Br` and `Wsr` are left out (a ditch is too narrow for an area), `Wp` running and
/// `Ws` standing water, `dr` road.
pub fn land_use_of_class(class: &str) -> Option<(&'static str, Option<&'static str>)> {
    match class.trim() {
        "Ls" => Some(("Wald", None)),
        "Lz" => Some(("Gehölz", None)),
        "Ł" | "Ps" => Some(("Landwirtschaft", Some("Grünland"))),
        "R" | "S" => Some(("Landwirtschaft", None)),
        "Wp" => Some(("Fließgewässer", None)),
        "Ws" => Some(("Stehendes Gewässer", None)),
        "dr" => Some(("Straßenverkehr", None)),
        _ => None,
    }
}

/// The land use of a parcel whose register classes are `classes` (`B,Bp`): that of its classes
/// when they all mean the same, else none. A parcel of arable land and a wood is neither.
fn parcel_land_use(classes: &str) -> Option<(&'static str, Option<&'static str>)> {
    let mut uses = classes.split(',').map(land_use_of_class);
    let first = uses.next()??;
    uses.all(|u| u == Some(first)).then_some(first)
}

/// The buildings as the engine reads them, in the CRS of the map.
pub fn buildings(features: &[Feature], crs: Crs) -> AlkisPart {
    let features = features
        .iter()
        .filter(|f| !f.id.is_empty())
        .filter_map(|f| {
            Some(json!({
                "type": "Feature",
                "id": f.id,
                "geometry": geometry(f, crs)?,
                "properties": {
                    "oid": f.id,
                    "gebnutzbez": "Gebäude",
                    "funktion": building_function(&f.kind),
                },
            }))
        })
        .collect();
    AlkisPart { features, crs }
}

/// The land use as the engine reads it, in the CRS of the map.
pub fn land_use(features: &[Feature], crs: Crs) -> AlkisPart {
    let features = features
        .iter()
        .filter(|f| !f.id.is_empty())
        .filter_map(|f| {
            let (nutzart, bez) = parcel_land_use(&f.kind)?;
            let mut properties = json!({"oid": f.id, "nutzart": nutzart});
            if let Some(bez) = bez {
                properties["bez"] = json!(bez);
            }
            Some(json!({
                "type": "Feature",
                "id": f.id,
                "geometry": geometry(f, crs)?,
                "properties": properties,
            }))
        })
        .collect();
    AlkisPart { features, crs }
}

/// The area as a box in EPSG:2180, or an error when it lies where no such position exists.
pub fn area_2180(area: &FetchArea) -> Result<BoundingBox, FetchError> {
    area_in_pl1992(area.crs, &area.bbox)
        .ok_or_else(|| http_error(ENDPOINT, "the map area has no position in the Polish grid"))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    const BUILDINGS: &str = include_str!("../tests/fixtures/egib/budynki-slubice.xml");
    const PARCELS: &str = include_str!("../tests/fixtures/egib/dzialki-slubice.xml");
    const URL: &str = "https://mapy.geoportal.gov.pl/x";

    /// Answers the requests in order and keeps their addresses.
    struct Pages {
        answers: RefCell<Vec<Vec<u8>>>,
        asked: RefCell<Vec<String>>,
    }

    impl Pages {
        fn new(answers: Vec<String>) -> Self {
            Self {
                answers: RefCell::new(answers.into_iter().rev().map(String::into_bytes).collect()),
                asked: RefCell::default(),
            }
        }
    }

    impl Fetcher for Pages {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.asked.borrow_mut().push(url.to_string());
            self.answers
                .borrow_mut()
                .pop()
                .ok_or_else(|| http_error(url, "no answer left"))
        }
    }

    /// A page of `count` buildings with the ids `<from>..`, naming `next` when given.
    fn page(from: usize, count: usize, next: Option<&str>) -> String {
        let next = next
            .map(|n| format!(r#" next="{}""#, n.replace('&', "&amp;")))
            .unwrap_or_default();
        let members: String = (from..from + count)
            .map(|i| {
                format!(
                    "<wfs:member><ms:budynki gml:id=\"b.{i}\"><ms:geom><gml:Polygon>\
                     <gml:exterior><gml:LinearRing><gml:posList>10 10 10 20 20 20 10 10\
                     </gml:posList></gml:LinearRing></gml:exterior></gml:Polygon></ms:geom>\
                     <ms:ID_BUDYNKU>B{i}</ms:ID_BUDYNKU><ms:RODZAJ>m</ms:RODZAJ></ms:budynki>\
                     </wfs:member>"
                )
            })
            .collect();
        format!(
            "<wfs:FeatureCollection xmlns:ms=\"m\" xmlns:gml=\"g\" xmlns:wfs=\"w\"{next}>{members}\
             </wfs:FeatureCollection>"
        )
    }

    fn slubice() -> BoundingBox {
        BoundingBox {
            min_e: 197_600.0,
            min_n: 507_400.2,
            max_e: 198_100.7,
            max_n: 507_900.0,
        }
    }

    #[test]
    fn the_box_is_sent_north_first_and_rounded_outward() {
        let url = first_url(Layer::Buildings, &slubice());
        assert!(url.starts_with(ENDPOINT), "{url}");
        assert!(url.contains("TYPENAMES=ms:budynki"), "{url}");
        assert!(url.contains("COUNT=1000"), "{url}");
        assert!(
            url.contains("BBOX=507400,197600,507900,198101,urn:ogc:def:crs:EPSG::2180"),
            "{url}"
        );
        assert!(first_url(Layer::Parcels, &slubice()).contains("TYPENAMES=ms:dzialki"));
    }

    #[test]
    fn a_page_of_buildings_is_read() {
        let page = read_page(Layer::Buildings, BUILDINGS.as_bytes(), URL).unwrap();
        assert_eq!(page.features.len(), 2);
        let first = &page.features[0];
        assert_eq!(first.id, "080505_4.0001.2682_BUD");
        assert_eq!(first.kind, "b");
        assert_eq!(first.polygons.len(), 1);
        // The pairs are north, east: Słubice lies near 507 000 north and 197 000 east.
        assert_eq!(first.polygons[0][0][0], (507_524.42, 197_783.855));
        assert_eq!(page.features[1].kind, "m");
        // The `next` link comes back with its ampersands, not as `&amp;`.
        let next = page.next.unwrap();
        assert!(
            next.contains("&STARTINDEX=2") || next.contains("&COUNT=2"),
            "{next}"
        );
        assert!(!next.contains("&amp;"), "{next}");
    }

    #[test]
    fn a_page_of_parcels_reads_the_classes() {
        let page = read_page(Layer::Parcels, PARCELS.as_bytes(), URL).unwrap();
        let kinds: Vec<_> = page.features.iter().map(|f| f.kind.as_str()).collect();
        assert_eq!(kinds, ["B", "dr", "B,Bp"]);
        assert!(page.features.iter().all(|f| !f.id.is_empty()));
        assert!(page.next.is_none());
    }

    #[test]
    fn a_ring_with_a_hole_keeps_both_rings() {
        let xml = "<wfs:FeatureCollection xmlns:ms=\"m\" xmlns:gml=\"g\" xmlns:wfs=\"w\">\
            <wfs:member><ms:dzialki><ms:ID_DZIALKI>D1</ms:ID_DZIALKI>\
            <ms:KLASOUZYTKI_EGIB>Ls</ms:KLASOUZYTKI_EGIB><ms:geom><gml:Polygon>\
            <gml:exterior><gml:LinearRing><gml:posList>0 0 0 9 9 9 0 0</gml:posList></gml:LinearRing></gml:exterior>\
            <gml:interior><gml:LinearRing><gml:posList>1 1 1 2 2 2 1 1</gml:posList></gml:LinearRing></gml:interior>\
            </gml:Polygon></ms:geom></ms:dzialki></wfs:member></wfs:FeatureCollection>";
        let page = read_page(Layer::Parcels, xml.as_bytes(), URL).unwrap();
        assert_eq!(page.features[0].polygons[0].len(), 2);
        assert_eq!(page.features[0].polygons[0][1][0], (1.0, 1.0));
    }

    #[test]
    fn answers_that_are_no_page_are_errors() {
        let report = "<ows:ExceptionReport xmlns:ows=\"o\"><ows:Exception exceptionCode=\"x\">\
                      <ows:ExceptionText>busy</ows:ExceptionText></ows:Exception></ows:ExceptionReport>";
        assert!(read_page(Layer::Buildings, report.as_bytes(), URL).is_err());
        let html = "<html><body>503</body></html>";
        assert!(read_page(Layer::Buildings, html.as_bytes(), URL).is_err());
        let cut = &page(0, 2, None)[..80];
        assert!(read_page(Layer::Buildings, cut.as_bytes(), URL).is_err());
        let odd = page(0, 1, None).replace("10 10 10 20", "10 10 10");
        assert!(read_page(Layer::Buildings, odd.as_bytes(), URL).is_err());
        let nan = page(0, 1, None).replace("10 10 10 20", "10 10 x 20");
        assert!(read_page(Layer::Buildings, nan.as_bytes(), URL).is_err());
    }

    #[test]
    fn a_full_page_asks_for_the_next_and_a_short_one_stops() {
        let next = format!("{ENDPOINT}?STARTINDEX=1000");
        let pages = Pages::new(vec![
            page(0, PAGE, Some(&next)),
            page(PAGE, 7, Some("never")),
        ]);
        let features = fetch_all(&pages, Layer::Buildings, &slubice()).unwrap();
        assert_eq!(features.len(), PAGE + 7);
        let asked = pages.asked.borrow();
        assert_eq!(asked.len(), 2);
        assert_eq!(asked[1], next);
    }

    #[test]
    fn a_short_page_does_not_follow_its_next_link() {
        let pages = Pages::new(vec![page(0, 3, Some("https://mapy.geoportal.gov.pl/more"))]);
        assert_eq!(
            fetch_all(&pages, Layer::Buildings, &slubice())
                .unwrap()
                .len(),
            3
        );
        assert_eq!(pages.asked.borrow().len(), 1);
    }

    #[test]
    fn a_page_that_names_itself_ends_the_read_and_doubles_are_kept_once() {
        let first = first_url(Layer::Buildings, &slubice());
        // The second page repeats the ids 998 and 999 of the first.
        let pages = Pages::new(vec![
            page(0, PAGE, Some(&format!("{ENDPOINT}?STARTINDEX=1000"))),
            page(PAGE - 2, PAGE, Some(&first)),
        ]);
        let features = fetch_all(&pages, Layer::Buildings, &slubice()).unwrap();
        assert_eq!(features.len(), PAGE + PAGE - 2);
        assert_eq!(pages.asked.borrow().len(), 2);
    }

    #[test]
    fn a_service_that_never_ends_is_stopped() {
        let answers = (0..MAX_PAGES + 1)
            .map(|i| {
                page(
                    i * PAGE,
                    PAGE,
                    Some(&format!("{ENDPOINT}?STARTINDEX={}", i * PAGE + PAGE)),
                )
            })
            .collect();
        let pages = Pages::new(answers);
        let error = fetch_all(&pages, Layer::Buildings, &slubice()).unwrap_err();
        assert!(error.to_string().contains("more than 200 pages"), "{error}");
        assert_eq!(pages.asked.borrow().len(), MAX_PAGES);
    }

    #[test]
    fn the_error_of_a_page_is_asked_again() {
        let report = "<ows:ExceptionReport xmlns:ows=\"o\"><ows:Exception/></ows:ExceptionReport>";
        let pages = Pages::new(vec![report.to_string(), page(0, 2, None)]);
        assert_eq!(
            fetch_all(&pages, Layer::Buildings, &slubice())
                .unwrap()
                .len(),
            2
        );
    }

    /// The inverse of the grid finds the position to a millimetre, in the middle and at the edges
    /// of Poland.
    #[test]
    fn the_grid_is_inverted_to_a_millimetre() {
        for (lat, lon) in [
            (52.349, 14.556),
            (52.23, 21.01),
            (49.2, 22.6),
            (54.8, 18.3),
            (50.2, 15.0),
        ] {
            let at = LatLon { lat, lon };
            let p = Pl1992::forward(at);
            let back = pl1992_inverse(p.easting, p.northing).unwrap();
            let again = Pl1992::forward(back);
            assert!((again.easting - p.easting).abs() < 1e-3, "{lat} {lon}");
            assert!((again.northing - p.northing).abs() < 1e-3, "{lat} {lon}");
        }
        assert!(pl1992_inverse(f64::NAN, 5e5).is_none());
    }

    /// A building of the fixture lies where Słubice is.
    #[test]
    fn a_building_lands_on_its_place_in_the_map_crs() {
        let page = read_page(Layer::Buildings, BUILDINGS.as_bytes(), URL).unwrap();
        let part = buildings(&page.features, Crs::Utm33);
        assert_eq!(part.crs, Crs::Utm33);
        assert_eq!(part.features.len(), 2);
        let feature = &part.features[0];
        assert_eq!(feature["properties"]["oid"], "080505_4.0001.2682_BUD");
        assert_eq!(feature["properties"]["gebnutzbez"], "Gebäude");
        assert_eq!(feature["properties"]["funktion"], "Bürogebäude");
        let [e, n] = [
            feature["geometry"]["coordinates"][0][0][0][0]
                .as_f64()
                .unwrap(),
            feature["geometry"]["coordinates"][0][0][0][1]
                .as_f64()
                .unwrap(),
        ];
        // The first corner, 507524.420 N 197783.855 E of the grid: back to the grid it is the same.
        let ll = Crs::Utm33
            .inverse(freemapper_georef::crs::ProjectedPoint {
                easting: e,
                northing: n,
            })
            .unwrap();
        let grid = Pl1992::forward(ll);
        assert!(
            (grid.northing - 507_524.42).abs() < 0.01 && (grid.easting - 197_783.855).abs() < 0.01
        );
        assert!(
            (52.34..52.36).contains(&ll.lat) && (14.55..14.57).contains(&ll.lon),
            "{ll:?}"
        );
    }

    #[test]
    fn the_kinds_of_buildings_have_names() {
        let table = [
            ("m", "Wohngebäude"),
            ("g", "Gebäude für Wirtschaft und Gewerbe"),
            ("p", "Industriegebäude"),
            ("t", "Gebäude für Verkehr und Nachrichtenwesen"),
            ("h", "Handels- und Dienstleistungsgebäude"),
            ("z", "Behälter, Silo und Speichergebäude"),
            ("b", "Bürogebäude"),
            ("s", "Gebäude für Gesundheit und Pflege"),
            ("o", "Gebäude für Bildung, Wissenschaft und Kultur"),
            ("k", "Gebäude für Sport"),
            ("i", "Sonstiges Gebäude"),
            ("q", "q"),
        ];
        for (kind, name) in table {
            assert_eq!(building_function(kind), name, "{kind}");
        }
    }

    #[test]
    fn the_classes_of_parcels_have_their_land_use() {
        let table = [
            ("Ls", Some(("Wald", None))),
            ("Lz", Some(("Gehölz", None))),
            ("Ł", Some(("Landwirtschaft", Some("Grünland")))),
            ("Ps", Some(("Landwirtschaft", Some("Grünland")))),
            ("R", Some(("Landwirtschaft", None))),
            ("S", Some(("Landwirtschaft", None))),
            ("Wp", Some(("Fließgewässer", None))),
            ("Ws", Some(("Stehendes Gewässer", None))),
            ("dr", Some(("Straßenverkehr", None))),
            ("B", None),
            ("Bp", None),
            ("N", None),
            ("Wsr", None),
            ("", None),
        ];
        for (class, expected) in table {
            assert_eq!(land_use_of_class(class), expected, "{class:?}");
        }
        assert_eq!(land_use_of_class(" Ls "), Some(("Wald", None)));
    }

    #[test]
    fn a_parcel_of_two_uses_has_none() {
        assert_eq!(parcel_land_use("Ls"), Some(("Wald", None)));
        assert_eq!(parcel_land_use("R,S"), Some(("Landwirtschaft", None)));
        assert_eq!(
            parcel_land_use("Ł,Ps"),
            Some(("Landwirtschaft", Some("Grünland")))
        );
        assert_eq!(parcel_land_use("R,Ls"), None);
        assert_eq!(parcel_land_use("Ls,B"), None);
        assert_eq!(parcel_land_use("B,Ls"), None);
        assert_eq!(parcel_land_use("B,Bp"), None);
        assert_eq!(parcel_land_use(""), None);
    }

    #[test]
    fn land_use_keeps_the_parcels_with_a_clear_use() {
        let page = read_page(Layer::Parcels, PARCELS.as_bytes(), URL).unwrap();
        let part = land_use(&page.features, Crs::Utm33);
        // Of the built-up parcel, the road and the parcel of two built-up classes only the road.
        assert_eq!(part.features.len(), 1);
        let properties = &part.features[0]["properties"];
        assert_eq!(properties["nutzart"], "Straßenverkehr");
        assert!(properties.get("bez").is_none());
        let meadow = Feature {
            id: "D9".into(),
            kind: "Ł".into(),
            polygons: page.features[0].polygons.clone(),
        };
        let part = land_use(&[meadow], Crs::Utm33);
        assert_eq!(part.features[0]["properties"]["bez"], "Grünland");
    }

    #[test]
    fn a_feature_without_id_or_position_is_left_out() {
        let polygons = vec![vec![vec![(507_000.0, 197_000.0), (507_010.0, 197_000.0)]]];
        let nameless = Feature {
            id: String::new(),
            kind: "m".into(),
            polygons: polygons.clone(),
        };
        assert!(buildings(&[nameless], Crs::Utm33).features.is_empty());
        let lost = Feature {
            id: "L".into(),
            kind: "m".into(),
            polygons: vec![vec![vec![(f64::NAN, 1.0)]]],
        };
        assert!(buildings(&[lost], Crs::Utm33).features.is_empty());
    }

    #[test]
    fn the_area_is_taken_into_the_polish_grid() {
        let area = FetchArea::new(
            BoundingBox {
                min_e: 470_000.0,
                min_n: 5_800_000.0,
                max_e: 470_500.0,
                max_n: 5_800_500.0,
            },
            Crs::Utm33,
        );
        let bbox = area_2180(&area).unwrap();
        assert!(bbox.min_n > 500_000.0 && bbox.min_n < 520_000.0, "{bbox:?}");
        assert!(bbox.width() > 500.0 && bbox.height() > 500.0);
    }

    /// The real service, for Słubice: `cargo test -p freemapper-adapters-pl -- --ignored egib`.
    #[test]
    #[ignore = "asks the national cadastre"]
    fn the_live_service_gives_buildings_and_parcels_for_slubice() {
        use crate::polska::Polska;
        use freemapper::fetch::HttpFetcher;
        use freemapper_adapter_api::adapter::RegionAdapter as _;
        use freemapper_adapter_api::policy::PolicyFetcher;
        let adapter = Polska::new(&[]);
        let http = HttpFetcher::new();
        let fetcher = PolicyFetcher::new(&http, adapter.policy().unwrap());
        // About a quarter of a square kilometre around the Oder bridge.
        let bbox = BoundingBox {
            min_e: 197_600.0,
            min_n: 507_400.0,
            max_e: 197_900.0,
            max_n: 507_700.0,
        };
        let buildings = fetch_all(&fetcher, Layer::Buildings, &bbox).unwrap();
        let parcels = fetch_all(&fetcher, Layer::Parcels, &bbox).unwrap();
        println!(
            "{} buildings, {} parcels; kinds {:?}",
            buildings.len(),
            parcels.len(),
            parcels
                .iter()
                .map(|p| p.kind.as_str())
                .collect::<std::collections::BTreeSet<_>>()
        );
        assert!(!buildings.is_empty() && !parcels.is_empty());
        assert!(!land_use(&parcels, Crs::Utm33).features.is_empty());
    }
}
