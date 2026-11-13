//! The buildings of a CityGML 1.0 file (the LoD2 tiles of the AdV profile), read as a stream:
//! of each building its outline at ground level, its ids, its function and the date of the
//! outline. Walls and roofs are passed over; a file is never held in memory as a whole.

use std::io::BufRead;

use crate::shapefile::Polygon;
use crate::xml::resolve_entity;

/// One `bldg:Building` with a ground surface.
#[derive(Debug, Clone, PartialEq)]
pub struct CityBuilding {
    /// `gml:id` of the `bldg:Building`.
    pub gml_id: String,
    /// `externalReference/externalObject/name`.
    pub alkis_id: Option<String>,
    /// `bldg:function` as written (`31001_2000`); of the first part when the building has none.
    pub function: Option<String>,
    /// The `gen:stringAttribute` named `Grundrissaktualitaet` (`2025-12-02`).
    pub outline_date: Option<String>,
    /// The polygons of every `bldg:GroundSurface` of the building and its parts: closed rings
    /// of `[x, y]`, the outer ring first, heights dropped.
    pub ground: Vec<Polygon>,
}

/// What a file held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CityRead {
    /// Buildings given to the caller.
    pub buildings: usize,
    /// Buildings without a ground surface, which were not given.
    pub without_ground: usize,
}

/// Calls `each` for every building with a ground surface, in file order.
///
/// Elements are matched by their local name, whatever the prefixes are. Points come from
/// `gml:posList` and `gml:pos`, three values each. A document that is not a `CityModel`, or
/// that ends before its root closes, is an error.
pub fn read_buildings<R: BufRead>(
    reader: R,
    each: &mut dyn FnMut(CityBuilding),
) -> Result<CityRead, String> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_reader(reader);
    let mut buffer = Vec::new();
    let mut read = CityRead::default();
    // Local names of the open elements, the root first.
    let mut path: Vec<String> = Vec::new();
    let mut root_closed = false;
    let mut draft: Option<Draft> = None;
    let mut text = String::new();
    loop {
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(|e| format!("CityGML: {e}"))?;
        let empty = matches!(event, Event::Empty(_));
        match event {
            Event::Start(e) | Event::Empty(e) => {
                let name = local(e.name().as_ref());
                if path.is_empty() {
                    if root_closed || name != "CityModel" {
                        return Err(format!("not a CityGML file: no CityModel but <{name}>"));
                    }
                    if empty {
                        root_closed = true;
                        continue;
                    }
                }
                text.clear();
                let attribute = |key: &str| {
                    e.attributes()
                        .flatten()
                        .find(|a| local(a.key.as_ref()) == key)
                        .map(|a| a.value.to_string())
                };
                match draft.as_mut() {
                    None if name == "Building" => {
                        draft = Some(Draft::new(attribute("id").unwrap_or_default()));
                    }
                    Some(d) => d.open(&name, attribute("name")),
                    None => {}
                }
                if empty {
                    // An element without content closes at once.
                    if let Some(d) = draft.as_mut() {
                        d.close(&name, path.last().map(String::as_str), "")?;
                    }
                } else {
                    path.push(name);
                }
            }
            Event::Text(t) => text.push_str(t.as_ref()),
            Event::GeneralRef(r) => text.extend(resolve_entity(r.as_ref())),
            Event::End(_) => {
                let Some(name) = path.pop() else { continue };
                if path.is_empty() {
                    root_closed = true;
                }
                if name == "Building" {
                    if let Some(done) = draft.take() {
                        match done.finish() {
                            Some(building) => {
                                read.buildings += 1;
                                each(building);
                            }
                            None => read.without_ground += 1,
                        }
                    }
                } else if let Some(d) = draft.as_mut() {
                    d.close(&name, path.last().map(String::as_str), text.trim())?;
                }
                text.clear();
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if !root_closed {
        return Err("not a whole CityGML file: it ends before its CityModel closes".to_string());
    }
    Ok(read)
}

/// The local name of an element or attribute: without its prefix.
fn local(name: &str) -> String {
    name.rsplit(':').next().unwrap_or_default().to_string()
}

/// The values of a point per the CityGML of the AdV: x, y and the height.
const DIMENSION: usize = 3;

/// A building while its element is open.
struct Draft {
    gml_id: String,
    alkis_id: Option<String>,
    function: Option<String>,
    part_function: Option<String>,
    outline_date: Option<String>,
    ground: Vec<Polygon>,
    /// Whether the open `stringAttribute` is the date of the outline.
    in_outline_date: bool,
    /// Open `GroundSurface` elements (never more than one in practice).
    in_ground: usize,
    /// The polygon of a ground surface that is open, its exterior first.
    polygon: Option<Polygon>,
    /// The ring that is open, and whether it is the exterior.
    ring: Vec<[f64; 2]>,
    exterior: bool,
}

impl Draft {
    fn new(gml_id: String) -> Self {
        Self {
            gml_id,
            alkis_id: None,
            function: None,
            part_function: None,
            outline_date: None,
            ground: Vec::new(),
            in_outline_date: false,
            in_ground: 0,
            polygon: None,
            ring: Vec::new(),
            exterior: true,
        }
    }

    /// An element named `name` opens inside the building; `attribute_name` is its `name`
    /// attribute.
    fn open(&mut self, name: &str, attribute_name: Option<String>) {
        match name {
            "stringAttribute" => {
                self.in_outline_date = attribute_name.as_deref() == Some("Grundrissaktualitaet");
            }
            "GroundSurface" => self.in_ground += 1,
            "Polygon" if self.in_ground > 0 => self.polygon = Some(Vec::new()),
            "exterior" => self.exterior = true,
            "interior" => self.exterior = false,
            "LinearRing" => self.ring.clear(),
            _ => {}
        }
    }

    /// The element `name` (a child of `parent`) closes with the text `text`.
    fn close(&mut self, name: &str, parent: Option<&str>, text: &str) -> Result<(), String> {
        match name {
            "name" if parent == Some("externalObject") => {
                if self.alkis_id.is_none() && !text.is_empty() {
                    self.alkis_id = Some(text.to_string());
                }
            }
            "value" if self.in_outline_date => {
                if self.outline_date.is_none() && !text.is_empty() {
                    self.outline_date = Some(text.to_string());
                }
            }
            "stringAttribute" => self.in_outline_date = false,
            "function" if !text.is_empty() => {
                let slot = if parent == Some("BuildingPart") {
                    &mut self.part_function
                } else {
                    &mut self.function
                };
                slot.get_or_insert_with(|| text.to_string());
            }
            "GroundSurface" => self.in_ground = self.in_ground.saturating_sub(1),
            "posList" | "pos" if self.polygon.is_some() => self.points(text)?,
            "LinearRing" => {
                let Some(polygon) = self.polygon.as_mut() else {
                    return Ok(());
                };
                let mut ring = std::mem::take(&mut self.ring);
                if ring.first() != ring.last() {
                    ring.push(ring[0]);
                }
                if ring.len() < 4 {
                    return Ok(());
                }
                if self.exterior {
                    polygon.insert(0, ring);
                } else {
                    polygon.push(ring);
                }
            }
            "Polygon" => {
                if let Some(polygon) = self.polygon.take()
                    && !polygon.is_empty()
                {
                    self.ground.push(polygon);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Adds the points of a point list (or of one `gml:pos`) to the open ring.
    fn points(&mut self, text: &str) -> Result<(), String> {
        let mut values = Vec::new();
        for word in text.split_ascii_whitespace() {
            values.push(word.parse::<f64>().map_err(|_| {
                format!(
                    "building {}: `{}` in a point list is no number",
                    self.gml_id,
                    crate::fetch::one_line(word, 20)
                )
            })?);
        }
        if values.len() % DIMENSION != 0 {
            return Err(format!(
                "building {}: a point list of {} values",
                self.gml_id,
                values.len()
            ));
        }
        self.ring.extend(
            values
                .as_chunks::<DIMENSION>()
                .0
                .iter()
                .map(|p| [p[0], p[1]]),
        );
        Ok(())
    }

    /// The building, if it has a ground surface.
    fn finish(self) -> Option<CityBuilding> {
        (!self.ground.is_empty()).then(|| CityBuilding {
            gml_id: self.gml_id,
            alkis_id: self.alkis_id,
            function: self.function.or(self.part_function),
            outline_date: self.outline_date,
            ground: self.ground,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    const FIXTURE: &str = include_str!("../../../tests/fixtures/bayern-lod2.gml");

    fn read(text: &str) -> Result<(CityRead, Vec<CityBuilding>), String> {
        let mut buildings = Vec::new();
        let read = read_buildings(text.as_bytes(), &mut |b| buildings.push(b))?;
        Ok((read, buildings))
    }

    /// A CityGML document with `members` inside its root.
    fn model(members: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<CityModel xmlns="http://www.opengis.net/citygml/1.0" xmlns:gml="http://www.opengis.net/gml"
 xmlns:bldg="http://www.opengis.net/citygml/building/1.0" xmlns:gen="http://www.opengis.net/citygml/generics/1.0">
{members}
</CityModel>"#
        )
    }

    /// A surface of `kind` (`GroundSurface`, `WallSurface`, …) with one polygon of `rings`
    /// (the first is the exterior), each a point list as written.
    fn surface(kind: &str, rings: &[&str]) -> String {
        let ring = |role: &str, points: &str| {
            format!(
                "<gml:{role}><gml:LinearRing><gml:posList>{points}</gml:posList></gml:LinearRing></gml:{role}>"
            )
        };
        let mut body = ring("exterior", rings[0]);
        for points in &rings[1..] {
            body.push_str(&ring("interior", points));
        }
        format!(
            "<bldg:boundedBy><bldg:{kind}><bldg:lod2MultiSurface><gml:MultiSurface srsDimension=\"3\">\
             <gml:surfaceMember><gml:Polygon>{body}</gml:Polygon></gml:surfaceMember>\
             </gml:MultiSurface></bldg:lod2MultiSurface></bldg:{kind}></bldg:boundedBy>"
        )
    }

    const SQUARE: &str = "0 0 5 0 10 5 10 10 5 10 0 5 0 0 5";
    const INNER: &str = "4 4 5 4 6 5 6 6 5 6 4 5 4 4 5";

    fn building(id: &str, body: &str) -> String {
        format!(
            "<cityObjectMember><bldg:Building gml:id=\"{id}\">{body}</bldg:Building></cityObjectMember>"
        )
    }

    #[test]
    fn a_building_gives_its_outline_ids_function_and_date() {
        let (read, buildings) = read(FIXTURE).unwrap();
        assert_eq!(
            read,
            CityRead {
                buildings: 3,
                without_ground: 0
            }
        );
        let first = &buildings[0];
        assert_eq!(first.gml_id, "DEBY_LOD2_6286165");
        assert_eq!(first.alkis_id.as_deref(), Some("DEBYvAAAAABA6CHp"));
        assert_eq!(first.function.as_deref(), Some("31001_2000"));
        assert_eq!(first.outline_date.as_deref(), Some("2026-01-09"));
        assert_eq!(first.ground.len(), 1);
        let ring = &first.ground[0][0];
        assert!(ring.len() >= 4);
        assert_eq!(ring.first(), ring.last());
        // The tile 726_5434 of EPSG:25832.
        for [x, y] in ring {
            assert!((726_000.0..=728_000.0).contains(x), "{x}");
            assert!((5_434_000.0..=5_436_000.0).contains(y), "{y}");
        }
        // The second building of the file has a ground surface with an inner ring, the
        // third is a roofed structure.
        assert_eq!(buildings[1].ground[0].len(), 2);
        assert_eq!(buildings[2].function.as_deref(), Some("51009_1610"));
        assert_eq!(buildings[2].alkis_id.as_deref(), Some("DEBYvAAAAAB3QcfM"));
    }

    #[test]
    fn wall_and_roof_surfaces_are_not_outlines() {
        let body = format!(
            "{}{}{}",
            surface("WallSurface", &["0 0 5 10 0 5 10 0 9 0 0 9 0 0 5"]),
            surface("GroundSurface", &[SQUARE]),
            surface("RoofSurface", &["0 0 9 10 0 9 10 10 9 0 10 9 0 0 9"]),
        );
        let (_, buildings) = read(&model(&building("b1", &body))).unwrap();
        assert_eq!(buildings[0].ground.len(), 1);
        assert_eq!(
            buildings[0].ground[0][0],
            [
                [0.0, 0.0],
                [0.0, 10.0],
                [10.0, 10.0],
                [10.0, 0.0],
                [0.0, 0.0]
            ]
        );
    }

    #[test]
    fn the_parts_of_a_building_add_their_outlines() {
        let part = |function: &str, ring: &str| {
            format!(
                "<bldg:consistsOfBuildingPart><bldg:BuildingPart>{function}{}</bldg:BuildingPart></bldg:consistsOfBuildingPart>",
                surface("GroundSurface", &[ring])
            )
        };
        let body = format!(
            "{}{}",
            part("<bldg:function>31001_1010</bldg:function>", SQUARE),
            part(
                "<bldg:function>31001_2463</bldg:function>",
                "20 0 5 30 0 5 30 10 5 20 10 5 20 0 5"
            ),
        );
        let (found, buildings) = read(&model(&building("b1", &body))).unwrap();
        assert_eq!(found.buildings, 1);
        assert_eq!(buildings[0].ground.len(), 2);
        assert_eq!(buildings[0].function.as_deref(), Some("31001_1010"));
        // The function of the building itself goes before that of a part.
        let own = format!("<bldg:function>31001_2000</bldg:function>{body}");
        let (_, buildings) = read(&model(&building("b1", &own))).unwrap();
        assert_eq!(buildings[0].function.as_deref(), Some("31001_2000"));
    }

    #[test]
    fn an_inner_ring_stays_a_hole() {
        let body = surface("GroundSurface", &[SQUARE, INNER]);
        let (_, buildings) = read(&model(&building("b1", &body))).unwrap();
        assert_eq!(buildings[0].ground[0].len(), 2);
        assert_eq!(buildings[0].ground[0][1][0], [4.0, 4.0]);
    }

    #[test]
    fn points_given_one_by_one_are_read() {
        let body = "<bldg:boundedBy><bldg:GroundSurface><bldg:lod2MultiSurface><gml:MultiSurface>\
            <gml:surfaceMember><gml:Polygon><gml:exterior><gml:LinearRing>\
            <gml:pos>0 0 5</gml:pos><gml:pos>10 0 5</gml:pos><gml:pos>10 10 5</gml:pos><gml:pos>0 0 5</gml:pos>\
            </gml:LinearRing></gml:exterior></gml:Polygon></gml:surfaceMember>\
            </gml:MultiSurface></bldg:lod2MultiSurface></bldg:GroundSurface></bldg:boundedBy>";
        let (_, buildings) = read(&model(&building("b1", body))).unwrap();
        assert_eq!(
            buildings[0].ground[0][0],
            [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 0.0]]
        );
    }

    #[test]
    fn a_building_without_a_ground_surface_is_counted() {
        let body = surface("RoofSurface", &[SQUARE]);
        let (read, buildings) = read(&model(&building("b1", &body))).unwrap();
        assert_eq!(
            read,
            CityRead {
                buildings: 0,
                without_ground: 1
            }
        );
        assert!(buildings.is_empty());
    }

    #[test]
    fn a_point_list_that_is_not_in_threes_names_the_building() {
        let body = surface("GroundSurface", &["0 0 10 0 10 10 0 0"]);
        let error = read(&model(&building("DEBY_LOD2_7", &body))).unwrap_err();
        assert!(error.contains("building DEBY_LOD2_7"), "{error}");
        assert!(error.contains("8 values"), "{error}");
        let body = surface("GroundSurface", &["0 0 5 x 0 5 10 10 5 0 0 5"]);
        let error = read(&model(&building("DEBY_LOD2_8", &body))).unwrap_err();
        assert!(error.contains("building DEBY_LOD2_8"), "{error}");
    }

    #[test]
    fn a_file_cut_short_is_an_error() {
        // After the second building: the root never closes.
        let end = FIXTURE
            .match_indices("</cityObjectMember>")
            .nth(1)
            .unwrap()
            .0;
        assert!(read(&FIXTURE[..end + "</cityObjectMember>".len()]).is_err());
        // In the middle of a tag.
        assert!(read(&FIXTURE[..end + 5]).is_err());
        // Not CityGML at all.
        let error = read("<html><body>503</body></html>").unwrap_err();
        assert!(error.contains("CityModel"), "{error}");
        assert!(read("").is_err());
    }

    #[test]
    fn the_reader_takes_a_stream() {
        let mut count = 0;
        let stream = BufReader::with_capacity(64, FIXTURE.as_bytes());
        let read = read_buildings(stream, &mut |_| count += 1).unwrap();
        assert_eq!(read.buildings, 3);
        assert_eq!(count, 3);
    }

    #[test]
    fn a_model_without_members_has_no_buildings() {
        // The form of a tile of forest or water, with the `core:` prefix on the root.
        let text = r#"<?xml version="1.0" encoding="UTF-8"?>
<core:CityModel xmlns:core="http://www.opengis.net/citygml/1.0" xmlns:gml="http://www.opengis.net/gml">
  <gml:description>building in LOD2</gml:description>
  <gml:name>LoD2_32_714_5332_2_BY</gml:name>
  <gml:boundedBy>
    <gml:Envelope srsName="urn:adv:crs:ETRS89_UTM32*DE_DHHN2016_NH">
      <gml:lowerCorner srsDimension="3">714000.000 5332000.000 0.000</gml:lowerCorner>
      <gml:upperCorner srsDimension="3">716000.000 5334000.000 0.000</gml:upperCorner>
    </gml:Envelope>
  </gml:boundedBy>
</core:CityModel>
"#;
        let (read, buildings) = read(text).unwrap();
        assert_eq!(read, CityRead::default());
        assert!(buildings.is_empty());
    }
}
