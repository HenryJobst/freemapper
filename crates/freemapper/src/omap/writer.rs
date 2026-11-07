//! XML writer for OOM map files.

use std::fmt::Write as _;

use quick_xml::escape::escape;

use super::OmapError;
use super::coord::MapCoord;
use super::model::{MapDocument, MapObject, MapObjectKind, TemplateKind, TemplateRef};

const NAMESPACE: &str = "http://openorienteering.org/apps/mapper/xml/v2";
const FILE_FORMAT_VERSION: u32 = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmlFormat {
    /// `.omap`: coordinates as compact text (`x y flags;`).
    Compact,
    /// `.xmap`-style: one `<coord>` element per coordinate.
    Pretty,
}

/// Writes `doc` as OOM XML (file format version 9).
pub fn write_map(
    doc: &MapDocument,
    mut out: impl std::io::Write,
    format: XmlFormat,
) -> Result<(), OmapError> {
    out.write_all(render(doc, format).as_bytes())?;
    Ok(())
}

fn render(doc: &MapDocument, format: XmlFormat) -> String {
    let mut s = String::new();
    let g = &doc.georef;
    let _ = writeln!(s, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    let _ = writeln!(
        s,
        r#"<map xmlns="{NAMESPACE}" version="{FILE_FORMAT_VERSION}">"#
    );
    let _ = writeln!(s, "<notes>{}</notes>", escape(&doc.notes));
    let _ = writeln!(
        s,
        r#"<georeferencing scale="{}" grid_scale_factor="{}" auxiliary_scale_factor="1" declination="{}" grivation="{}">"#,
        g.scale_denominator,
        g.grid_scale_factor,
        g.declination_deg,
        g.grivation_deg()
    );
    let _ = writeln!(
        s,
        r#"<projected_crs id="EPSG"><spec language="PROJ.4">+init=epsg:{epsg}</spec><parameter>{epsg}</parameter><ref_point x="{}" y="{}"/></projected_crs>"#,
        g.ref_projected.easting,
        g.ref_projected.northing,
        epsg = g.epsg
    );
    let _ = writeln!(
        s,
        r#"<geographic_crs id="Geographic coordinates"><spec language="PROJ.4">+proj=latlong +datum=WGS84</spec><ref_point_deg lat="{}" lon="{}"/></geographic_crs>"#,
        g.ref_geographic.lat, g.ref_geographic.lon
    );
    s.push_str("</georeferencing>\n");
    let _ = writeln!(s, "{}", doc.symbol_set.colors_xml);
    s.push_str("<barrier version=\"6\" required=\"0.6.0\">\n");
    let _ = writeln!(s, "{}", doc.symbol_set.symbols_xml);
    write_parts(&mut s, doc, format);
    write_templates(&mut s, &doc.templates);
    write_view(&mut s, doc);
    s.push_str("</barrier>\n</map>\n");
    s
}

fn write_parts(s: &mut String, doc: &MapDocument, format: XmlFormat) {
    let default_part = [super::model::MapPart {
        name: "default part".to_string(),
        objects: Vec::new(),
    }];
    let parts = if doc.parts.is_empty() {
        &default_part[..]
    } else {
        &doc.parts[..]
    };
    let _ = writeln!(s, r#"<parts count="{}" current="0">"#, parts.len());
    for part in parts {
        let _ = writeln!(
            s,
            r#"<part name="{}"><objects count="{}">"#,
            escape(&part.name),
            part.objects.len()
        );
        for object in &part.objects {
            write_object(s, object, format);
        }
        s.push_str("</objects></part>\n");
    }
    s.push_str("</parts>\n");
}

fn write_object(s: &mut String, object: &MapObject, format: XmlFormat) {
    match &object.kind {
        MapObjectKind::Point { rotation_rad } => {
            let _ = write!(
                s,
                r#"<object type="0" symbol="{}" rotation="{}">"#,
                object.symbol_id, rotation_rad
            );
        }
        MapObjectKind::Path => {
            let _ = write!(s, r#"<object type="1" symbol="{}">"#, object.symbol_id);
        }
        MapObjectKind::Text {
            h_align, v_align, ..
        } => {
            let _ = write!(
                s,
                r#"<object type="4" symbol="{}" h_align="{h_align}" v_align="{v_align}">"#,
                object.symbol_id
            );
        }
    }
    if !object.tags.is_empty() {
        s.push_str("<tags>");
        for (k, v) in &object.tags {
            let _ = write!(s, r#"<t k="{}">{}</t>"#, escape(k), escape(v));
        }
        s.push_str("</tags>");
    }
    write_coords(s, &object.coords, format);
    match &object.kind {
        MapObjectKind::Path => {
            s.push_str(r#"<pattern rotation="0"><coord x="0" y="0"/></pattern>"#);
        }
        MapObjectKind::Text { text, .. } => {
            let _ = write!(s, "<text>{}</text>", escape(text));
        }
        MapObjectKind::Point { .. } => {}
    }
    s.push_str("</object>\n");
}

fn write_coords(s: &mut String, coords: &[MapCoord], format: XmlFormat) {
    let _ = write!(s, r#"<coords count="{}">"#, coords.len());
    for c in coords {
        match (format, c.flags) {
            (XmlFormat::Compact, 0) => {
                let _ = write!(s, "{} {};", c.x, c.y);
            }
            (XmlFormat::Compact, flags) => {
                let _ = write!(s, "{} {} {flags};", c.x, c.y);
            }
            (XmlFormat::Pretty, 0) => {
                let _ = write!(s, "\n<coord x=\"{}\" y=\"{}\"/>", c.x, c.y);
            }
            (XmlFormat::Pretty, flags) => {
                let _ = write!(
                    s,
                    "\n<coord x=\"{}\" y=\"{}\" flags=\"{flags}\"/>",
                    c.x, c.y
                );
            }
        }
    }
    if format == XmlFormat::Pretty {
        s.push('\n');
    }
    s.push_str("</coords>");
}

fn template_type(kind: TemplateKind) -> &'static str {
    match kind {
        TemplateKind::Image => "TemplateImage",
        TemplateKind::Gdal => "GdalTemplate",
        TemplateKind::Ogr => "OgrTemplate",
        TemplateKind::Map => "TemplateMap",
    }
}

/// Templates in file order: the ones behind the map first, each group in the given order.
fn ordered_templates(templates: &[TemplateRef]) -> Vec<&TemplateRef> {
    let (behind, front): (Vec<_>, Vec<_>) = templates.iter().partition(|t| t.behind_map);
    behind.into_iter().chain(front).collect()
}

fn write_templates(s: &mut String, templates: &[TemplateRef]) {
    let _ = writeln!(
        s,
        r#"<templates count="{}" first_front_template="{}">"#,
        templates.len(),
        templates.iter().filter(|t| t.behind_map).count()
    );
    for t in ordered_templates(templates) {
        let name = t.relpath.rsplit('/').next().unwrap_or(&t.relpath);
        let _ = write!(
            s,
            r#"<template type="{}" open="true" name="{}" path="{}" relpath="{}" georef="{}">"#,
            template_type(t.kind),
            escape(name),
            escape(&t.relpath),
            escape(&t.relpath),
            t.georeferenced
        );
        if let Some(crs) = &t.crs_spec {
            let _ = write!(s, "<crs_spec>{}</crs_spec>", escape(crs));
        }
        s.push_str("</template>\n");
    }
    s.push_str(
        "<defaults use_meters_per_pixel=\"true\" meters_per_pixel=\"0\" dpi=\"0\" scale=\"0\"/>\n",
    );
    s.push_str("</templates>\n");
}

fn write_view(s: &mut String, doc: &MapDocument) {
    s.push_str("<view>\n");
    if !doc.symbol_set.grid_xml.is_empty() {
        let _ = writeln!(s, "{}", doc.symbol_set.grid_xml);
    }
    s.push_str(
        r#"<map_view zoom="1" position_x="0" position_y="0"><map opacity="1" visible="true"/>"#,
    );
    if doc.templates.is_empty() {
        s.push_str(r#"<templates count="0"/>"#);
    } else {
        let _ = write!(s, r#"<templates count="{}">"#, doc.templates.len());
        for (i, t) in ordered_templates(&doc.templates).into_iter().enumerate() {
            let _ = write!(
                s,
                r#"<ref template="{i}" visible="{}" opacity="{}"/>"#,
                t.visible, t.opacity
            );
        }
        s.push_str("</templates>");
    }
    s.push_str("</map_view>\n</view>\n");
}
