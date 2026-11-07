use freemapper::crs::{EPSG_UTM33, LatLon, ProjectedPoint};
use freemapper::georef::Georeferencing;
use freemapper::omap::{
    MapCoord, MapDocument, MapObject, MapObjectKind, MapPart, SymbolSet, TemplateKind, TemplateRef,
    XmlFormat, closed_ring, write_map,
};

const TEST_SET: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<map xmlns="http://openorienteering.org/apps/mapper/xml/v2" version="9">
<georeferencing scale="4000"><projected_crs id="Local coordinates"/></georeferencing>
<colors count="1">
<color priority="0" name="Black" c="0" m="0" y="0" k="1" opacity="1"><cmyk method="custom"/><rgb method="custom" r="0" g="0" b="0"/></color>
</colors>
<barrier version="6" required="0.6.0">
<symbols count="1" id="Test">
<symbol type="4" id="0" code="1" name="Area"><area_symbol inner_color="0" min_area="0" patterns="0"/></symbol>
</symbols>
<view>
<grid color="#646464" display="0" alignment="0" additional_rotation="0" unit="1" h_spacing="500" v_spacing="500" h_offset="0" v_offset="0" snapping_enabled="true"/>
</view>
</barrier>
</map>
"##;

fn georef() -> Georeferencing {
    Georeferencing {
        scale_denominator: 1000,
        epsg: EPSG_UTM33,
        ref_geographic: LatLon {
            lat: 52.5401,
            lon: 13.8442,
        },
        ref_projected: ProjectedPoint {
            easting: 421_600.0,
            northing: 5_821_700.0,
        },
        declination_deg: 5.5,
        convergence_deg: -0.92,
        grid_scale_factor: 0.999675,
    }
}

fn doc(notes: &str, templates: Vec<TemplateRef>) -> MapDocument {
    MapDocument {
        notes: notes.to_string(),
        georef: georef(),
        symbol_set: SymbolSet::from_omap_xml(TEST_SET).unwrap(),
        parts: vec![MapPart {
            name: "default part".to_string(),
            objects: vec![MapObject {
                symbol_id: 0,
                kind: MapObjectKind::Path,
                coords: closed_ring(&[(0, 0), (4000, 0), (4000, -2000), (0, -2000)]),
                tags: vec![("freemapper:source".to_string(), "test".to_string())],
            }],
        }],
        templates,
    }
}

fn render(doc: &MapDocument, format: XmlFormat) -> String {
    let mut out = Vec::new();
    write_map(doc, &mut out, format).unwrap();
    String::from_utf8(out).unwrap()
}

fn parses(xml: &str) {
    let mut reader = quick_xml::Reader::from_str(xml);
    loop {
        match reader.read_event().unwrap() {
            quick_xml::events::Event::Eof => break,
            _ => continue,
        }
    }
}

#[test]
fn closed_ring_appends_flagged_first_point() {
    let ring = closed_ring(&[(0, 0), (10, 0), (10, 10)]);
    assert_eq!(ring.len(), 4);
    assert_eq!(ring[0], MapCoord::new(0, 0));
    assert_eq!(
        ring[3],
        MapCoord {
            x: 0,
            y: 0,
            flags: 18
        }
    );
}

#[test]
fn closed_ring_dedups_closed_input() {
    let ring = closed_ring(&[(0, 0), (10, 0), (10, 10), (0, 0)]);
    assert_eq!(ring.len(), 4);
    assert_eq!(ring[2], MapCoord::new(10, 10));
    assert_eq!(ring[3].flags, 18);
}

#[test]
fn golden_minimal_pretty() {
    let xml = render(&doc("test notes", vec![]), XmlFormat::Pretty);
    let expected = include_str!("golden/minimal.xmap");
    assert_eq!(xml, expected);
}

#[test]
fn compact_coords_are_text() {
    let xml = render(&doc("n", vec![]), XmlFormat::Compact);
    assert!(
        xml.contains(r#"<coords count="5">0 0;4000 0;4000 -2000;0 -2000;0 0 18;</coords>"#),
        "{xml}"
    );
    parses(&xml);
}

#[test]
fn notes_are_utf8_escaped() {
    let xml = render(
        &doc(
            "© GeoBasis-DE/LGB 2026, dl-de/by-2-0, Daten geändert <x>",
            vec![],
        ),
        XmlFormat::Compact,
    );
    assert!(xml.contains("© GeoBasis-DE/LGB 2026, dl-de/by-2-0, Daten geändert &lt;x&gt;"));
    parses(&xml);
}

#[test]
fn template_refs_visible_in_view() {
    let template = TemplateRef {
        kind: TemplateKind::Map,
        relpath: "templates/candidates.omap".to_string(),
        georeferenced: true,
        crs_spec: None,
        visible: true,
        opacity: 0.7,
        behind_map: false,
    };
    let xml = render(&doc("n", vec![template]), XmlFormat::Compact);
    assert!(xml.contains(r#"type="TemplateMap""#), "{xml}");
    assert!(xml.contains(r#"relpath="templates/candidates.omap""#));
    assert!(xml.contains(r#"georef="true""#));
    assert!(xml.contains(r#"<templates count="1" first_front_template="0">"#));
    assert!(xml.contains(r#"<ref template="0" visible="true" opacity="0.7"/>"#));
    parses(&xml);
}

fn image_template(relpath: &str, visible: bool) -> TemplateRef {
    TemplateRef {
        kind: TemplateKind::Image,
        relpath: relpath.to_string(),
        georeferenced: true,
        crs_spec: Some("+init=epsg:25833".to_string()),
        visible,
        opacity: 0.5,
        behind_map: true,
    }
}

#[test]
fn behind_templates_come_first() {
    let candidates = TemplateRef {
        kind: TemplateKind::Map,
        relpath: "templates/candidates.omap".to_string(),
        georeferenced: true,
        crs_spec: None,
        visible: true,
        opacity: 0.7,
        behind_map: false,
    };
    let xml = render(
        &doc(
            "n",
            vec![candidates, image_template("templates/hillshade.png", false)],
        ),
        XmlFormat::Compact,
    );
    assert!(
        xml.contains(r#"<templates count="2" first_front_template="1">"#),
        "{xml}"
    );
    let image = xml.find(r#"relpath="templates/hillshade.png""#).unwrap();
    let map = xml.find(r#"relpath="templates/candidates.omap""#).unwrap();
    assert!(image < map, "behind-map template is written first");
    assert!(xml.contains(
        r#"<ref template="0" visible="false" opacity="0.5"/><ref template="1" visible="true" opacity="0.7"/>"#
    ));
    parses(&xml);
}

#[test]
fn image_template_has_crs_spec() {
    let xml = render(
        &doc("n", vec![image_template("templates/orthophoto.jpg", true)]),
        XmlFormat::Compact,
    );
    assert!(
        xml.contains(r#"<template type="TemplateImage" open="true" name="orthophoto.jpg""#),
        "{xml}"
    );
    assert!(xml.contains(r#"georef="true"><crs_spec>+init=epsg:25833</crs_spec></template>"#));
    parses(&xml);
}

fn text_doc(text: &str) -> MapDocument {
    let mut d = doc("n", Vec::new());
    d.parts[0].objects = vec![MapObject {
        symbol_id: 0,
        kind: MapObjectKind::Text {
            text: text.to_string(),
            h_align: 1,
            v_align: 0,
        },
        coords: vec![MapCoord::new(1000, 2000)],
        tags: Vec::new(),
    }];
    d
}

#[test]
fn text_object_written_like_oom() {
    let xml = render(&text_doc("Clara-Zetkin-Schule"), XmlFormat::Compact);
    assert!(
        xml.contains(r#"<object type="4" symbol="0" h_align="1" v_align="0"><coords count="1">1000 2000;</coords><text>Clara-Zetkin-Schule</text></object>"#),
        "{xml}"
    );
    parses(&xml);
}

#[test]
fn text_object_escapes_content() {
    let xml = render(&text_doc(r#"A & B <C> "D""#), XmlFormat::Compact);
    assert!(
        xml.contains("<text>A &amp; B &lt;C&gt; &quot;D&quot;</text>"),
        "{xml}"
    );
    parses(&xml);
}

#[test]
fn georef_writes_the_project_epsg() {
    let mut zone32 = doc("", Vec::new());
    zone32.georef.epsg = 25832;
    let xml = render(&zone32, XmlFormat::Pretty);
    assert!(
        xml.contains("<parameter>25832</parameter>") && xml.contains("+init=epsg:25832"),
        "{xml}"
    );
    parses(&xml);
}
